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
use std::collections::HashMap;
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
        None => { println!("Started on localhost:3001");
                  "localhost:3001".to_string() },
    };

    let listener = TcpListener::bind(web_address).unwrap();
    for stream in listener.incoming().by_ref() {
        match stream {
            Ok(mut stream) => {
                spawn( move || {
                    match Request::parse_stream(&mut stream) {
                        Ok(request) => {
                            let mut response: Response;
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
        None => "123",
    };
    let mut response = Status::ok();
    
    let mut body_work = String::from_utf8(START_PAGE.to_vec()).unwrap()
                        .replace("{user_id}", user_id);

    response.body(body_work.as_bytes().to_vec());
    
    let body_len = &response.body_len().to_string();
    response.header("Content-Length", body_len);
    response
}

fn handle_tictac(request: &Request) -> Response {
    //
    let user_id = match request.body.as_ref() {
        Some(ref map) => map.get("user_id").unwrap(),
        None => "123", // TODO - fetch unused id from game server
    };
    let move_to = match request.body {
        Some(ref map) =>  map.get("move_to").unwrap().as_bytes()[0] as char,
        None => '_',
    };
    let new_game = match request.body {
        Some(ref map) => { match map.get("new_game").unwrap().as_str() {
                        "false" => false,
                        "true" => true,
                        _ => false,
                       }
                     },
        None => true,
    };
    
    let user_data = UserData {
                        user_id:user_id.parse::<u32>().unwrap(),
                        move_to:move_to,
                        new_game:new_game
                    };
    
    let mut stream_to_game =
        match TcpStream::connect("127.0.0.1:3001") {
            Ok(_) => {},
            Err(e) => println!("Game server down? {:?}",e),
        };
    
    let mut response = Status::ok();
    
    let mut body_work = String::from_utf8(GAME_PAGE.to_vec()).unwrap()
                        .replace("{user_id}", user_id);
    
    response.body( body_work.as_bytes().to_vec() );
    
    let body_len = &response.body_len().to_string();
    response.header("Content-Length", body_len);
    response
}

