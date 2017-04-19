extern crate rustc_serialize;

use ::Msg::*;
use std::io::{Read, Write, Bytes};
use std::str;
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::collections::HashMap;
use std::net::{TcpListener, SocketAddr, TcpStream};

// This is a library consisting of all data structs and/or
// funcitonality shared between the web and game servers

// outgoing and incoming data is parsed to this via JSON
#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct UserData {
    pub user_id : u32,
    pub move_to : char,
    pub new_game: bool,
}

// TODO use the url_map to parse the url string query
pub struct Request {
    pub method : String,
    pub url    : String,
    pub headers: HashMap<String, String>,
    pub body   : Option<HashMap<String, String>>, // TODO make enum so can use a HashMap or Vec
}
impl Request {
    pub fn new() -> Request {
        Request {
            method  : String::new(),
            url     : String::new(),
            headers : HashMap::new(),
            body    : None,
        }
    }
}

pub fn parse_stream(stream: &mut TcpStream) -> Result<Request, &'static str> {
    //stream.set_read_timeout(None).expect("set_read_timeout call failed");
    //stream.set_write_timeout(None).expect("set_write_timeout call failed");
    stream.set_ttl(100).expect("set_ttl call failed");

    let mut read_len = 0;
    let mut buffer:[u8; 2048] = [0; 2048]; // limit helps avoid swamping the server. 2048 is typical

    let mut req = Request::new();
    let mut body = String::new();
    let mut key = String::new();
    let mut val = String::new();

    let mut state = 0;
    let mut method_state = 0;
    let mut url_state = 0;
    let mut headers_state = 0;
    let mut body_state = 0;

    // Begin state machine - run until buffer cleared
    match stream.read(&mut buffer) {
        Ok(len) => read_len = len,
        Err(e) => {}
    }
    for n in 0..read_len {
        let c = buffer[n] as char;
        match state {
            // Method
            0 => {
                match method_state {
                    0 => {
                        if c == ' ' { method_state = 1; }
                        else { req.method.push(c); }
                    },
                    1 => {
                        if c == ' ' { method_state = 2; }
                        else { req.url.push(c); }
                    },
                    2 => {
                        if c == '\r' { method_state = 3; }
                        // TODO - process url params
                    }, // Ignore HTTP version for now
                    3 => {
                        if c == '\n' { state = 1; }
                        else {
                            return Err("Server unable to parse request method");
                        }
                    },
                    _ => {},
                }
            }
            // Headers
            1 => {
                match headers_state {
                    // Start of line
                    0 => {
                        if c == ' ' { headers_state = 1; }
                        else { key.push(c); }
                    },
                    // Space encountered // remove the ':'
                    1 => {
                        headers_state = 2;
                        key.pop();
                        val.push(c); }
                    // Get key value
                    2 => {
                        if c == '\r' { headers_state = 3 }
                        else { val.push(c); }
                    },
                    // got '\r' value, c is now '\n', ignore it
                    3 => headers_state = 4,
                    // end of line or body?
                    4 => {
                        if c == '\r' { headers_state = 5; }
                        else {
                            headers_state = 0;
                            req.headers.insert(key, val);
                            key = String::new();
                            val = String::new();
                            key.push(c);
                        }
                    },
                    // c is now '\n' - end of headers
                    5 => state = 2,
                    _ => {},
                }
            },
            // Body state
            2 => {
                    body.push(c);
            },
            _ => {},
        }
    }

    if body.len() > 0 {
        req.body = Some(parse_params(&body));
    }
    Ok(req)
}

fn parse_params(string: &String) -> HashMap<String,String> {
    let mut map:HashMap<String,String> = HashMap::new();
    // Make an array of strings plit by '&'
    let pairs:Vec<&str> = string.split('&').collect();
    // For each string, split by '=' to key/val pair
    for pair in pairs {
        let keyval:Vec<&str> = pair.split('=').collect();
        map.insert(keyval[0].to_string(),keyval[1].to_string());
    }
    map
}

pub struct Response {
    pub code   : String,
    pub headers: HashMap<String, String>,
    pub body   : Option<Vec<u8>>,
}
impl Response {
    pub fn new() -> Response {
        Response { code: String::new(),
                   headers: HashMap::new(),
                   body: None, }
    }
    pub fn status(&mut self, status: &str, expl: Option<&str>) {
        let ver = "HTTP/1.1 ".to_string();
        self.code = ver+status+" "+expl.unwrap_or("");
    }
    pub fn header(&mut self, key: &str, val: &str) {
        self.headers.insert(key.to_string(),val.to_string());
    }
    pub fn body(&mut self, text: Vec<u8>) {
        self.body = Some(text);
    }
}
impl ToString for Response {
    fn to_string(&self) -> String {
        let line_end = String::from("\r\n");
        let mut string: String = self.code.to_string() + &line_end;
        for (key, val) in &self.headers {
            string += &(key.clone() + ": " + val + &line_end);
        }
        match self.body {
            Some(ref b) => {string.push_str(&line_end);
                            string.push_str(str::from_utf8(&b).unwrap());
                            string.push_str(&line_end)},
            None => {},
        }
        string
    }
}

pub mod Msg {
    use ::Response;
    use std::collections::HashMap;

    pub fn connection_error() -> Response {
        let mut res = Response::new();
        res.status("520", Some("Unkown Error"));
        res.header("Content-Type", "text/html");
        res
    }
    pub fn ok() -> Response {
        let mut res = Response::new();
        res.status("200", Some("Ok"));
        res.header("Content-Type", "text/html");
        res
    }
    pub fn bad_request() -> Response {
        let mut res = Response::new();
        res.status("400", Some("Bad Request"));
        res.header("Content-Type", "text/html");
        res
    }
    pub fn not_found()  -> Response {
        let mut res = Response::new();
        res.status("404", Some("Not Found"));
        res.header("Content-Type", "text/html");
        res
    }
    pub fn internal_error() -> Response {
        let mut res = Response::new();
        res.status("500", Some("Internal Server Error"));
        res.header("Content-Type", "text/html");
        res
    }
    pub fn not_implemented() -> Response {
        let mut res = Response::new();
        res.status("501", Some("Request Not Implemented"));
        res.header("Content-Type", "text/html");
        res
    }
}
