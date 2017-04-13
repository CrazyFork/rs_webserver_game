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

struct Request {
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
fn parse_stream(stream: &TcpStream) -> Request {
        Request {
                method: String::from("GET"),
                url: String::from("/"),
                headers: HashMap::new(),
                body: None,
        }
    }

type Handler = Box<Fn(&Request) -> Response>;
/// The server stores a Handler function (callback or closure)
/// which takes a Request and produces a Response.
/// The server parses the incoming connections to a Request
/// and shunts this off to the Handler - the returned Response
/// is then sent to the client
pub fn start_server(addr: String) -> JoinHandle<()> {
    // Spawn a thread to handle incoming streams so we aren't blocking
    // TODO: Return JoinHandle in a Result
    spawn( || {
        let listener = TcpListener::bind(addr).unwrap();
        for stream in listener.incoming().by_ref() {
            match stream {
                Ok(mut stream) => {
                    spawn( move || {
                        let request = parse_stream(&stream);
                        let response = connection_error();// UrlMap(request);
                        stream.write_all(response.to_string().as_bytes()).unwrap();
                    });
                }
                Err(e) => { println!("Bad connection: {:?}", e); }
            }
        }
    })
}

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
