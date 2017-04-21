#[macro_use]
extern crate lazy_static;
extern crate rustc_serialize;
extern crate common;

use common::{Request, Response, UserData, Status};
use rustc_serialize::json;
use std::env;
use std::io::{Read,Write};
use std::fs::File;
use std::thread::spawn; // spawning threads
use std::net::{TcpListener, TcpStream, Shutdown};

lazy_static!{
    static ref START_PAGE: Vec<u8> = {
        let mut file = File::open("index.html").expect("index.html not found");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("Error reading index.html");
        buf
    };
}
lazy_static!{
    static ref GAME_PAGE: Vec<u8> = {
        let mut file = File::open("game.html").expect("game.html not found");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("Error reading game.html");
        buf
    };
}

fn main() {
    let mut args = env::args(); // args is an iter
    // Using .nth() discards all preceding elements in the iterator
    let web_address = match args.nth(1) {
        Some(s) => s,
        None => { println!("Started on localhost:3000");
                  "localhost:3000".to_string() },
    };

    let listener = TcpListener::bind(web_address).unwrap();
    for stream in listener.incoming().by_ref() {
        match stream {
            Ok(mut stream) => {
                spawn( move || {
                    match Request::parse_stream(&mut stream) {
                        Ok(request) => {
                            let response: Response;
                            match request.url.as_ref() {
                                "/" =>      response = handle_new(&request),
                                "/game/" => response = handle_tictac(&request),
                                _ =>        response = Status::not_found(),
                            }
                            stream.write_all(response.to_string().as_bytes()).unwrap();
                            stream.shutdown(Shutdown::Both).unwrap();
                        },
                        Err(e) => stream.write_all(e.as_bytes()).unwrap(),
                    }
                });
            }
            Err(e) => { println!("Bad connection: {:?}", e); }
        }
    }
}

/// Handle new users via url '/'
///
/// The Response here is a page that contains a button
/// to start a new game - if there is no user_id in the
/// request then one is randomly generated and inserted
/// in to the response.
fn handle_new(request: &Request) -> Response {
    let ref user_id = match request.body.as_ref() {
        Some(map) => map.get("user_id").unwrap(),
        None => "123",// TODO - fetch unused id from game server
    };
    let mut response = Status::ok();
    
    let body_work = String::from_utf8(START_PAGE.to_vec()).unwrap()
                        .replace("{user_id}", user_id);

    response.body(body_work.as_bytes().to_vec());
    
    let body_len = &response.body_len().to_string();
    response.header("Content-Length", body_len);
    response
}

fn handle_tictac(request: &Request) -> Response {
    // Request body is optional, need to check it exists first
    // the .get_param() will return a Response to use if Err
    let user_id = match request.get_param("user_id") {
        Ok(user) => user,
        Err(e) => return e,
    };
    let move_to = match request.get_param("move_to") {
        Ok(mv) => mv.as_bytes()[0] as char,
        Err(_) => 'n',
    };
    // We can get around this typing stuff by using an enum storage for the body hashmap
    // ... Maybe later
    let new_game = match request.get_param("new_game") {
        Ok(ng) => {
            match ng.as_str() {
                "false" => false,
                "true" => true,
                _ => false,
            }
        },
        Err(_) => true, // Maybe shouldn't ignore the error, but the other fields are fine
    };
    
    let user_data = UserData {
                        user_id: user_id.parse::<u32>().unwrap(),
                        move_to: move_to,
                        new_game:new_game
                    };
    let user_json = match json::encode(&user_data) {
            Err(e) => format!("JSON conversion failed: {:?}", e),
            Ok(o) => {
                if move_to == 'n' {
                    "1:".to_string()+&o
                } else {
                    "0:".to_string()+&o
                }
            }
    };
    
    // Send JSON to game_server and parse recd JSON to data structure (vec)
    let game = match rw_user_data(&user_json, "localhost:3001") {
        Ok(game) => {
            match json::decode(&String::from_utf8(game).unwrap()) {
                Err(_) => return Status::internal_error(),
                Ok(o) => o,
            }
        },
        Err(_) => return Status::internal_error(),
    };
    
    // Create the html table
    let game_table = create_table(game);
    
    let mut response = Status::ok();
    let body_work = String::from_utf8(GAME_PAGE.to_vec()).unwrap()
                        .replace("{user_id}", user_id)
                        .replace("{game_table}", &game_table);
    
    response.body( body_work.as_bytes().to_vec() );
    
    let body_len = &response.body_len().to_string();
    response.header("Content-Length", body_len);
    response
}

fn rw_user_data(user_data: &str, addr: &str) -> Result<Vec<u8>, String> {
    let mut game:Vec<u8> = Vec::new();
    match TcpStream::connect(addr) {
            Ok(ref mut o) => {
                o.write_all(user_data.as_bytes()).unwrap();
                o.shutdown(Shutdown::Write).unwrap();
                o.read_to_end(&mut game).unwrap();
                return Ok(game)
            },
            Err(e) => return Err(format!("Game server down? {:?}",e)),
    };
}

fn create_table(game: Vec<Vec<char>>) -> String {
    let mut game_table = String::new();
    for row in game {
        game_table.push_str("<tr>");
        for col in row {
            game_table.push_str("<td>[");
            game_table.push(col);
            game_table.push_str("]</td>");
        }
        game_table.push_str("</tr>");
    }
    println!("{:?}", game_table);
    game_table
}

