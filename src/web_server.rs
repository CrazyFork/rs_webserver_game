extern crate rustc_serialize;
extern crate common;

use common::Status;
use common::{Request, Response};
use rustc_serialize::json;
use std::env;
use std::io::Write;
use std::thread::spawn; // spawning threads
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, Shutdown};

fn main() {
    let mut args = env::args(); // args is an iter
    
    // Using .nth() discards all preceding elements in the iterator
    let web_address = match args.nth(1) {
        Some(s) => s,
        None => { println!("Usage: web_server <web_ip>:<port>"); return },
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
                                "/" => response = handle_new(&request),
                                "/game" => response = handle_tictac(&request),
                                _ => response = Status::internal_error(),
                            }
                            println!("Sent response: {:?}", response.to_string());
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

fn handle_new(request: &Request) -> Response {
    let ref user_id = match request.body.as_ref() {
        Some(map) => map.get("user_id").unwrap(),
        None => "123",
    };
    let mut response = Status::ok();
    response.body(
        format!("<h1>Test</h1><body>{:?}</body>", user_id)
        .as_bytes()
        .to_vec()
    );
    response
}

fn handle_tictac(request: &Request) -> Response {
    //
    let mut stream_to_game =
        match TcpStream::connect("127.0.0.1:3001") {
            Ok(_) => {},
            Err(e) => println!("Game server down? {:?}",e),
        };

    let mut response = Status::ok();
    response.body( "TEST".as_bytes().to_vec() );
    response
}

