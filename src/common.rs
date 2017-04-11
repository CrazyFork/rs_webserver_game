extern crate rustc_serialize;

use std::collections::HashMap;

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
    body   : Vec<u8>,
}

struct Response {
    code   : u32,
    headers: HashMap<String, String>,
    body   : Vec<u8>,
}

type BoxedFunc = Box<Fn(&Request) -> Response>;
/// The UrlMap stores callbacks for urls in a HashMap.
/// A callback must take a reference to a Request struct
/// and return a Response. The Response is sent to the client
struct UrlMap {
    maps: HashMap<String, BoxedFunc>,
}
impl<> UrlMap {
    fn new() -> UrlMap {
        UrlMap {
            maps: HashMap::new(),
        }
    }
    fn add(&mut self, url: String, func: BoxedFunc) {
        self.maps.insert(url, func);
    }
    fn handle(&self, request: &Request) -> Response {
        match self.maps.get(&request.url) {
            Some(c) => c(request),
            None => Response {
                        code: 404,
                        headers: HashMap::new(),
                        body: "404 - Not found".as_bytes().to_vec(),
                    },
        }
    }
}
