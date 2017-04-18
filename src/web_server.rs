extern crate rustc_serialize;
extern crate common;

use common::{Request, Response, parse_stream};
use rustc_serialize::json;
use std::env;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex}; // for safely threading
use std::thread::spawn; // spawning threads
use std::collections::HashMap;
use std::net::{TcpListener, SocketAddr, TcpStream, Shutdown};

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
                    match parse_stream(&stream) {
                        Ok(request) => {
                            let response = handle_tictac(&request);
                            println!("Sent response: {:?}", response.to_string());
                            stream.write_all(response.to_string().as_bytes()).unwrap();
                            stream.shutdown(Shutdown::Both).expect("shutdown call failed");
                        },
                        Err(e) => stream.write_all(e.as_bytes()).unwrap(),
                    }
                });
            }
            Err(e) => { println!("Bad connection: {:?}", e); }
        }
    }
}

fn handle_tictac(request: &Request) -> Response {
    //
    //let mut stream_to_game = TcpStream::connect("127.0.0.1:3001").unwrap();
    let mut headers:HashMap<String, String> = HashMap::new();
    headers.insert("Content-Type".to_string(), "text/html".to_string());
    Response {
        code: "HTTP/1.1 200 OK".to_string(),
        headers: headers,
        body: Some("TEST".as_bytes().to_vec()),
    }
}

