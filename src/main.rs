extern crate iron;
extern crate router;
extern crate urlencoded;
#[macro_use] extern crate mime;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::str::FromStr;
use std::io::Read;
use urlencoded::UrlEncodedBody;
use urlencoded::UrlEncodedQuery;

fn main() {
    let mut router = Router::new();
    router.get("/", start_game);
    router.post("/game", post_move);

    println!("Serving from localhost:3000");
    Iron::new(router).http("localhost:3000").unwrap();
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
    
    let input;
    match req.get::<UrlEncodedBody>() {
        Err(ref e) => {
            response.set_mut(status::BadRequest);
            response.set_mut(format!("Error parsing form data: {:?}\n", e));
            return Ok(response);
        }
        Ok(map) => { input = map; }
    }
    println!("input = {:?}", input);
    let url_data;
    match req.get::<UrlEncodedQuery>() {
        Err(ref e) => {
            response.set_mut(status::BadRequest);
            response.set_mut(format!("Error parsing url params: {:?}\n", e));
            return Ok(response);
        }
        Ok(map) => { url_data = map; }
    }
    println!("url_data = {:?}", url_data);

    //
    let user_move;
    match input.get("place") {
        None => {
            response.set_mut(mime!(Text/Html; Charset=Utf8));
            response.set_mut(r#"
            <title>Tic Tac Toe</title>
            <form action="/game?id=22&0=&1=&2=&3=&4=&5=&6=&7=&8=&9=" method="post" enctype='text/plain'>
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
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    response.set_mut(r#"
        <title>Tic Tac Toe</title>
        <form action="/game?id=22&0=&1=&2=&3=&4=&5=&6=&7=&8=&9=" method="post" enctype='text/plain'>
            <input type="text" name="place"/>
            <button type="submit">Move</button>
        </form>
    "#);
    Ok(response)
}
fn make_board(req: &mut Request) -> IronResult<Response> {
    println!("Success?");
    let mut response = Response::new();
    let url_data;
    match req.get_ref::<UrlEncodedQuery>() {
        Err(ref e) => {
            println!("{:?}", e);
            response.set_mut(status::BadRequest);
            response.set_mut(format!("Error parsing url params: {:?}\n", e));
            return Ok(response);
        }
        Ok(ref map) => { url_data = map.clone(); }
    }
    println!("Data = {:?}", url_data);
    response.set_mut(status::Ok);
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    response.set_mut(r#"
        <title>Tic Tac Toe</title>
        <form action="/game?id=22&0=&1=&2=&3=&4=&5=&6=&7=&8=&9=" method="post" enctype='text/plain'>
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
