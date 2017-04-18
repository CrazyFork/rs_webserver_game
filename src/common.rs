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

pub fn parse_stream(stream: &mut TcpStream) -> Result<Request, &'static str> {
    //stream.set_read_timeout(None).expect("set_read_timeout call failed");
    //stream.set_write_timeout(None).expect("set_write_timeout call failed");
    stream.set_ttl(100).expect("set_ttl call failed");

    let mut read_len = 0;
    let mut buffer:[u8; 2048] = [0; 2048]; // limit helps avoid swamping the server. 2048 is typical

    let mut method = String::new();
    let mut url = String::new();
    let mut headers:HashMap<String,String> = HashMap::new();
    let mut body = String::new();
    let mut body_map:HashMap<String,String> = HashMap::new();

    let mut state = 0;
    let mut method_state = 0;
    let mut url_state = 0;
    let mut headers_state = 0;
    let mut body_state = 0;
    let mut key = String::new();
    let mut val = String::new();

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
                    0 => { if c == ' ' { method_state = 1; }
                            else { method.push(c); }
                    },
                    1 => { if c == ' ' { method_state = 2; }
                            else { url.push(c); }
                    },
                    2 => { if c == '\r' { method_state = 3; }
                    }, // Ignore HTTP version for now
                    3 => { if c == '\n' { state = 1; }
                            else { return Err("Method parse failed"); }
                    },
                    _ => {},
                }
            }
            // Headers
            1 => {
                match headers_state {
                    // Start of line
                    0 => { if c == ' ' { headers_state = 1; }
                            else { key.push(c); }
                    },
                    // Space encountered // remove the ':'
                    1 => { headers_state = 2;
                           key.pop();
                           val.push(c); }
                    // Get key value
                    2 => { if c == '\r' { headers_state = 3 }
                            else { val.push(c); }
                    },
                    // got '\r' value, c is now '\n', ignore it
                    3 => headers_state = 4,
                    // end of line or body?
                    4 => {
                        if c == '\r' {
                            headers_state = 5;
                        } else {
                            headers_state = 0;
                            headers.insert(key, val);
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
        body_map = parse_params(&body);
    }
    for (key,val) in body_map { println!("{:?}: {:?}",key,val);}

    Ok(Request {
        method: method,
        url: url,
        headers: headers,
        body: None,
    })
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

type Handler = Box<Fn(&Request) -> Response>;

/// The UrlMap stores callbacks for urls in a HashMap.
/// A callback must take a reference to a Request struct
/// and return a Response. The Response is sent to the client
struct UrlMap {
    maps: HashMap<String, Handler>,
}
impl<> UrlMap {
    fn new() -> UrlMap {
        UrlMap {
            maps: HashMap::new(),
        }
    }
    fn add(&mut self, url: String, func: Handler) {
        self.maps.insert(url, func);
    }
    fn handle(&self, request: &Request) -> Response {
        match self.maps.get(&request.url) {
            Some(c) => c(request),
            None => Response { // safeguard response
                        code: "HTTP/1.1 404 Not Found".to_string(),
                        headers: HashMap::new(),
                        body: Some("404 - Not found".as_bytes().to_vec()),
                    },
        }
    }
}

mod Msg {
    use ::Response;
    use std::collections::HashMap;
    
    pub fn connection_error() -> Response {
        Response { code: "HTTP/1.1 666".to_string(),
                   headers: HashMap::new(),
                   body: Some("Connection Error".as_bytes().to_vec()), }
    }
}
