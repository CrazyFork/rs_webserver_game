extern crate rustc_serialize;

use std::io::Read;
use std::str;
use std::collections::HashMap;
use std::net::TcpStream;

// This is a library consisting of all data structs and/or
// funcitonality shared between the web and game servers

/// Outgoing and incoming data is parsed to this via JSON
#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct UserData {
    pub user_id : u32,
    pub move_to : char,
    pub new_game: bool,
}
#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Grid {
    data: Vec<Vec<char>>
}

enum State { Method, Headers, Body }
enum MState { Method, Url, Http, End }
enum HState { Key, Space, Value, Ret, Insert, End }

/// Incoming streams should be parsed to this struct
pub struct Request {
    pub method : String,
    pub url    : String,
    pub headers: HashMap<String, String>,
    pub body   : Option<HashMap<String, String>>, // TODO make enum so can use a HashMap or Vec
}
impl Request {
    /// Produce a blank Request
    pub fn new() -> Request {
        Request {
            method  : String::new(),
            url     : String::new(),
            headers : HashMap::new(),
            body    : None,
        }
    }
    /// fetch any param in the request body, return either a ref to the
    /// string, or a Response that can be used if desired.
    ///
    pub fn get_param(&self, param: &str) -> Result<&String, Response> {
        match self.body {
            Some(ref map) => {
                match map.get(param) {
                    Some(p) => {
                        if p.is_empty() {
                            return Err(Status::faulty_query(&format!("{:?} was empty", param)))
                        }
                        return Ok(p)
                    },
                    None => return Err(Status::faulty_query(&format!("No {:?} submitted in query", param)))
                };
            },
            None => {
                return Err(Status::faulty_query("No data submitted in query"))
            },
        };
    }
    /// Parse any stream of bytes (u8) in to a Request if the stream is valid
    ///
    /// example:
    ///     let listener = TcpListener::bind("localhost:3000").unwrap();
    ///     for stream in listener {
    ///         let request = parse_stream(&mut stream).unwrap();
    ///     }
    ///
    pub fn parse_stream(stream: &mut TcpStream) -> Result<Request, &str> {

        let mut buffer:[u8; 2048] = [0; 2048]; // limit helps avoid swamping the server. 2048 is typical

        let mut req = Request::new();
        let mut body = String::new();
        let mut url = String::new();
        let mut key = String::new();
        let mut val = String::new();

        let mut state = State::Method;
        let mut method_state = MState::Method;
        let mut headers_state = HState::Key;

        // Begin state machine - run until buffer cleared
        // The read function fills the provided buffer (and won't overrun), then
        // returns the length read
        let read_len = match stream.read(&mut buffer) {
            Ok(len) => len,
            Err(_) => return Err("Error reading stream"),
        };
        for n in 0..read_len {
            let c = buffer[n] as char; // convert byte to char
            match state {
                State::Method => {
                    match method_state {
                        // Fetch method
                        MState::Method => {
                            if c == ' ' {
                                method_state = MState::Url;
                            } else {
                                req.method.push(c);
                            }
                        },
                        MState::Url => {
                            if c == ' ' {
                                println!("Url string = {:?}", url);
                                method_state = MState::Http;
                            } else {
                                url.push(c);
                            }
                        },
                        // Ignore HTTP version for now
                        MState::Http => {
                            if c == '\r' { method_state = MState::End; }
                        },
                        _ => {
                            if c == '\n' {
                                state = State::Headers;
                                let url_split:Vec<&str> = url.split('?').collect();
                                if url_split.len() > 1 {
                                    req.body = Some(parse_params(&url_split[1].to_string()));
                                }
                                req.url = url_split[0].to_string();
                                println!("Url string = {:?}", req.url);
                            }
                            else {
                                return Err("Server unable to parse request method");
                            }
                        },
                    }
                }
                State::Headers => {
                    match headers_state {
                        // Start of line
                        HState::Key => {
                            if c == ' ' {
                                headers_state = HState::Space;
                            } else {
                                key.push(c);
                            }
                        },
                        // Space encountered - remove the ':'
                        HState::Space => {
                            headers_state = HState::Value;
                            key.pop();
                            val.push(c); } //TODO check this
                        HState::Value => {
                            if c == '\r' {
                                headers_state = HState::Ret
                            } else {
                                val.push(c);
                            }
                        },
                        // got '\r' value, c is now '\n', ignore it
                        HState::Ret => headers_state = HState::Insert,
                        // end of line or body?
                        HState::Insert => {
                            if c == '\r' {
                                headers_state = HState::End;
                            } else {
                                headers_state = HState::Key;
                            }
                            req.headers.insert(key, val);
                            key = String::new();
                            val = String::new();
                            key.push(c);
                        },
                        // c is now '\n' - end of headers
                        // HTTP spec says if a body is sent with a GET request,
                        // it should be ignored.
                        HState::End => {
                            if req.method == "POST" {
                                state = State::Body;
                            } else {
                                break;
                            }
                        },
                    }
                },
                State::Body => {
                        body.push(c);
                },
            }
        }

        if body.len() > 0 {
            // Append
            if let Some(existing) = req.body.as_mut() { // Option[T].as_mut()->Option(&mut T)
                for (key,val) in parse_params(&body) {
                    existing.insert(key,val);
                }
            }
            // Or replace if None
            // Has to be done this way as req.body is borrowed as mut above
            if req.body == None {
                req.body = Some(parse_params(&body));
            }
        }
        Ok(req)
    }
}

/// A helper function to parse params recieved in either URL
/// or body requests
///
/// example:
///    parse_params("user_id=123&place=3".to_string());
///
fn parse_params(string: &String) -> HashMap<String,String> {
    let mut map:HashMap<String,String> = HashMap::new();
    // Make an array of strings plit by '&'
    let pairs:Vec<&str> = string.split('&').collect();
    // For each string, split by '=' to key/val pair
    for pair in pairs {
        println!("{:?}", pair);
        let keyval:Vec<&str> = pair.split('=').collect();
        if keyval.len() == 2 {
           map.insert(keyval[0].to_string(),keyval[1].to_string());
        } else {
            println!("Malformed params encountered");
        }
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
    pub fn body_len(&self) -> u32 {
        match self.body.as_ref() {
            Some(body) => body.len() as u32,
            None => 0,
        }
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
pub mod Status {
    use ::Response;
    
    pub fn faulty_query(text: &str) -> Response {
        let mut res = Response::new();
        res.status("422", Some("Unprocessable Entity"));
        res.header("Content-Type", "text/html");
        res.body(text.as_bytes().to_vec());
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
    pub fn unkown_error() -> Response {
        let mut res = Response::new();
        res.status("520", Some("Unkown Error"));
        res.header("Content-Type", "text/html");
        res
    }
}
