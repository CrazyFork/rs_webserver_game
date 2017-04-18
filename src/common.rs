extern crate rustc_serialize;

use ::Msg::*;
use std::io::{Write, Bytes};
use std::str::FromStr;
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

pub fn parse_stream(stream: &TcpStream) -> Request {
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
    for n in 0..read_len {
        let c = buffer[n] as char;
        match state {
            // Method
            0 => {
                if c == ' ' {
                    method_state += 1;
                } else if c == '\r' {
                    method_state = 2;
                }
                match method_state {
                    0 => method.push(c),
                    1 => url.push(c),
                    // Going to skip getting the HTTP version for now
                    _ => {
                        if c == '\n' && buffer[n-1] as char == '\r' {
                            state = 1;
                        } else {
                            println!("Something wrong here [1]");
                        }
                    },
                }
            }
            // Headers
            1 => {
                if c == ' ' {
                    headers_state = 1;
                } else if c == '\r' {
                    headers_state = 3;
                }
                match headers_state {
                    // Start of line
                    0 => key.push(c),
                    // Space encountered
                    1 => {
                        key.pop(); // remove the ':'
                        headers_state = 2; }
                    // Get key value
                    2 => val.push(c),
                    // Save the key/val pair
                    3 => {
                        if c == '\n' {
                            if buffer[n+1] as char == '\r' && buffer[n+2] as char == '\n' {
                                state = 3;
                            } else {
                                headers_state = 0;
                            }
                        }
                        headers.insert(key, val);
                        key = String::new();
                        val = String::new();
                    },
                    _ => {},
                }
            },
            // Body state
            2 => {
                if n == read_len {
                    // Make an array of strings plit by '&'
                    let pairs = body.split('&');
                    // For each string, split by '=' to key/val pair
                    for pair in pairs {
                        let keyval:Vec<&str> = pair.split('=').collect();
                        body_map.insert(keyval[0].to_string(),keyval[1].to_string());
                    }
                } else if c != '\r' && c != '\n' {
                    body.push(c);
                }
            },
            _ => {},
        }
    }

    // Unless the stream or something else errored out, Request should be guaranteed
    Request {
        method: method,
        url: url,
        headers: headers,
        body: None,
    }
}

pub struct Response {
    pub code   : u32,
    pub headers: HashMap<String, String>,
    pub body   : Option<Vec<u8>>,
}
impl ToString for Response {
    fn to_string(&self) -> String {
        let line_end = String::from("\r\n");
        let header_end = String::from("\r\n\r\n");
        let mut string: String = self.code.to_string() + &line_end;
        for (key, val) in &self.headers {
            string += &(key.clone() + ": " + val + &line_end);
        }
        match self.body {
            Some(ref b) => string += &((header_end) + &String::from_utf8(b.clone()).unwrap()),
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
                        code: 404,
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
        Response { code: 666,
                   headers: HashMap::new(),
                   body: Some("Connection Error".as_bytes().to_vec()), }
    }
}
