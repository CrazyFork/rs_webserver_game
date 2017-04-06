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
