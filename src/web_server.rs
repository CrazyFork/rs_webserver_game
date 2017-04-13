extern crate rustc_serialize;
extern crate common;

use common::{Request, Response, parse_stream};
use rustc_serialize::json;
use std::env;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex}; // for safely threading
use std::thread::spawn; // spawning threads
use std::collections::HashMap;
use std::net::{TcpListener, SocketAddr, TcpStream};

fn main() {
    let mut args = env::args(); // args is an iter
    
    // Using .nth() discards all preceding elements in the iterator
    let web_address = match args.nth(1) {
        Some(s) => s,
        None => { println!("Usage: web_server <web_ip>:<port> <game_ip>:<port>"); return },
    };
    let game_address = match args.next() {
        Some(s) => s,
        None => { println!("Usage: web_server <web_ip>:<port> <game_ip>:<port>"); return },
    };
    let listener = TcpListener::bind(web_address).unwrap();

    let server = spawn( || {
        let listener = TcpListener::bind(web_address).unwrap();
        for stream in listener.incoming().by_ref() {
            match stream {
                Ok(mut stream) => {
                    spawn( move || {
                        let request = parse_stream(&stream);
                        let response = handle_tictac(&request, &game_address);
                        stream.write_all(response.to_string().as_bytes()).unwrap();
                    });
                }
                Err(e) => { println!("Bad connection: {:?}", e); }
            }
        }
    });
}

fn handle_tictac(request: &Request, game_addr: &String) -> Response {
    //
}

