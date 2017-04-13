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
    pub move_to : i32,
    pub new_game: bool,
}

pub struct Request {
    method : String,
    url    : String,
    headers: HashMap<String, String>,
    body   : Option<Vec<u8>>,
}

pub struct Response {
    code   : u32,
    headers: HashMap<String, String>,
    body   : Option<Vec<u8>>,
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
pub fn parse_stream(stream: &TcpStream) -> Request {
    Request {
            method: String::from("GET"),
            url: String::from("/"),
            headers: HashMap::new(),
            body: None,
    }
/*     //stream.set_read_timeout(None).expect("set_read_timeout call failed");
    //stream.set_write_timeout(None).expect("set_write_timeout call failed");
    stream.set_ttl(100).expect("set_ttl call failed");
    
    let mut header = String::new();
    let mut content = String::new();
    let mut content_len = 0;
    let mut char_count = 0;
    let mut head = true;
    for byte in Read::by_ref(&mut stream).bytes() {
        let c = byte.unwrap() as char;
        if head {
            header.push(c);
        } else if char_count < content_len {
            char_count += 1;
            content.push(c);
        } else {
            break;
        }
        if header.ends_with("\r\n\r\n") && head {
            head = false;
        }
    }
    for line in header.lines() { println!("{:?}", line)}
    for line in content.lines() { println!("{:?}", line)}
*/
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
