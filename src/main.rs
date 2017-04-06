extern crate iron;
extern crate router;
extern crate params;
#[macro_use] extern crate mime;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::str::FromStr;
use std::io::Read;
use params::{Params, Value};

fn main() {
    let mut html_router = Router::new();
    html_router.get("/", start_game, "start");
    html_router.post("/game", post_move, "move");
    println!("Serving HTML from localhost:3000");
    Iron::new(html_router).http("localhost:3000").unwrap();
}

#[allow(unused_variables)]
fn start_game(request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();

    response.set_mut(status::Ok);
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    //TODO: Need to set a unique user ID
    response.set_mut(r#"
        <title>Tic Tac Toe</title>
        <form action="/game?id=22" method="post">
            <input type="submit" name="start_game" value="Start Game"/>
        </form>
    "#);
    Ok(response)
}

fn post_move(req: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();
    // params crate puts UrlEncodedBody and UrlParameters in same map. Convienient!
    let map = req.get_ref::<Params>().unwrap();
    println!("params = {:?}", map);

    let user_move;
    match map.find(&["place"]) {
        None => {
            response.set_mut(status::Ok);
            response.set_mut(mime!(Text/Html; Charset=Utf8));
            response.set_mut(r#"
            <title>Tic Tac Toe</title>
            <form action="/game?id=22" method="post" enctype='text/plain'>
                <input type="text" name="place"/>
                <button type="submit">Move</button>
            </form>
            "#);
            return Ok(response);
        }
        Some(_move) => { user_move = _move}
    }
    println!("user move = {:?}", user_move);

    response.set_mut(status::Ok);
    // Mime crate makes creation of mime types fast and simple with a macro
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    response.set_mut(r#"
        <title>Tic Tac Toe</title>
        <form action="/game?id=22&cpu=1,3&user=2,7" method="post" enctype='text/plain'>
            <input type="text" name="place"/>
            <button type="submit">Move</button>
        </form>
    "#);
    Ok(response)
}

fn end_game(request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();
    Ok(response)
}

/*// The Dump -- Notes and testing stuff
fn main() {
    let mut args = env::args(); // args is an iter

    let address = match args.nth(1) {
        Some(s) => s.parse::<SocketAddr>().unwrap(),
        None => { println!("Usage: game_server <ip>:<port>");
                  return },
    };

    let listener = TcpListener::bind(address).unwrap();
    println!("Game server listens on : {:?}", address);
    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // closure that calls a func to operate on the stream
                spawn(|| handle_client(stream));
            }
            Err(e) => { println!("Bad connection: {:?}", e); }
        }
    }
}
fn ok() -> Vec<u8> {
    return String::from("HTTP/1.1 200 OK\r\n").into_bytes()
}
fn mime_text() -> Vec<u8> {
    return String::from("Content-Type: text/html; charset=utf-8\r\n").into_bytes()
}
// Take stream and convert to 
fn handle_client(mut stream: TcpStream) {
    println!("Handling incoming connection");
    //stream.set_read_timeout(None).expect("set_read_timeout call failed");
    //stream.set_write_timeout(None).expect("set_write_timeout call failed");
    stream.set_ttl(100).expect("set_ttl call failed");

    let mut buffer = String::new();
    //let mut buf: Buf = [0u8, ..10240]; // big enough?
    for byte in Read::by_ref(&mut stream).bytes() {
        let c = byte.unwrap() as char;
        buffer.push(c);
        if buffer.ends_with("\r\n\r\n") { break }
    }
    println!("Buffer = {:?}", buffer);
    stream.write_all(&ok()).unwrap();
    stream.write_all(&mime_text()).unwrap();
    let test = (r#"
        <title>Tic Tac Toe</title>
        <form action="/game?id=22&cpu=1,3&user=2,7" method="post" enctype='text/plain'>
            <input type="text" name="place"/>
            <button type="submit">Move</button>
        </form>
    "#).as_bytes();
    stream.write_all(test).unwrap();
}
//*/
