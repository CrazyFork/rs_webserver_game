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

/// Outgoing and incoming data is parsed to this via JSON
#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct UserData {
    pub user_id : u32,
    pub move_to : char,
    pub new_game: bool,
}

/// Incoming streams should be parsed to this struct
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

    pub fn parse_stream(stream: &mut TcpStream) -> Result<Request, &'static str> {
        //stream.set_read_timeout(None).expect("set_read_timeout call failed");
        //stream.set_write_timeout(None).expect("set_write_timeout call failed");
        stream.set_ttl(100).expect("set_ttl call failed");

        let mut read_len = 0;
        let mut buffer:[u8; 2048] = [0; 2048]; // limit helps avoid swamping the server. 2048 is typical

        let mut req = Request::new();
        let mut body = String::new();
        let mut url = String::new();
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
                        // Fetch method
                        0 => {
                            if c == ' ' { method_state = 1; }
                            else { req.method.push(c); }
                        },
                        // Url string + query
                        1 => {
                            if c == ' ' { method_state = 2; }
                            else { url.push(c); }
                        },
                        // Ignore HTTP version for now
                        2 => {
                            if c == '\r' { method_state = 3; }
                        },
                        3 => {
                            if c == '\n' {
                                state = 1;
                                let url_split:Vec<&str> = url.split('?').collect();
                                if url_split.len() > 1 {
                                    req.body = Some(parse_params(&url_split[1].to_string()));
                                }
                                req.url = url_split[0].to_string();
                            }
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
            if let Some(existing) = req.body.as_mut() {
                for (key,val) in parse_params(&body) {
                    existing.insert(key,val);
                }
            }
            if req.body == None {
                req.body = Some(parse_params(&body));
            }
        }
        Ok(req)
    }
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

/// A standard structure for the server Response
///
/// A new response will be blank, so the methods
/// `status`, `header`, and `body` need to be used
/// to insert content.
///
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
    
    /// Modify the status string with a status code and optional message
    ///
    /// example:
    ///    let mut response = Response::new();
    ///    response.status("200", Some("Ok"));
    ///
    pub fn status(&mut self, status: &str, expl: Option<&str>) {
        let ver = "HTTP/1.1 ".to_string();
        self.code = ver+status+" "+expl.unwrap_or("");
    }
    
    /// Insert headers in the format of key, value
    ///
    /// example:
    ///    let mut response = Response::new();
    ///    response.header("Content-Type", "text/html");
    ///
    pub fn header(&mut self, key: &str, val: &str) {
        self.headers.insert(key.to_string(),val.to_string());
    }
    
    /// Add/Remove a body from the Response
    ///
    /// example: 
    ///    let mut response = Response::new();
    ///    response.body = Some("TEST".as_bytes().to_vec());
    ///    response.body = None;
    ///
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

/// Some standard templates for responses.
/// Most won't need to be edited except for cases
/// where a body may be desirable, or it's an Ok.
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
