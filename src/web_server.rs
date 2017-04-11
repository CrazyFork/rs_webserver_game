extern crate rustc_serialize;

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

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // closure that calls a func to operate on the stream
                // for each client stream, we need to parse to a "Request"
                // and then we can handle it
                spawn(|| handle_client(stream));
            }
            Err(e) => { println!("Bad connection: {:?}", e); }
        }
    }
}

fn read_stream(mut stream: TcpStream) {
    //
}

/// Parse the incoming stream in to a Request;
///     struct Request {
///        method : String,
///        url    : String,
///        headers: HashMap<String, String>,
///        body   : Vec<u8>,
///    }
fn handle_client(mut stream: TcpStream) {
    //stream.set_read_timeout(None).expect("set_read_timeout call failed");
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
}
