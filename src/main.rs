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
    router.post("/*", post_move);

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
        <form action="/game/?id=22" method="post">
            <input type="submit" name="start_game" value="Start Game"/>
        </form>
    "#);
    Ok(response)
}

fn post_move(req: &mut Request) -> IronResult<Response> {
    println!("Success?");
    let mut response = Response::new();
    let input_map;
    match req.get_ref::<UrlEncodedBody>() {
        Err(ref e) => {
            println!("{:?}", e);
            response.set_mut(status::BadRequest);
            response.set_mut(format!("Error parsing form data: {:?}\n", e));
            return Ok(response);
        }
        Ok(map) => { input_map = map; }
    }

    //
    let user_move;
    match input_map.get("place") {
        None => {
            response.set_mut(mime!(Text/Html; Charset=Utf8));
            response.set_mut(r#"
                <title>Tic Tac Toe</title>
                <form action="/game/?id=22&X=4&O=3" method="post" enctype='text/plain'>
                    <input type="submit" name="make_move" value="Move"/>
                </form>
            "#);
            return Ok(response);
        }
        Some(_move) => { user_move = _move}
    }

    response.set_mut(status::Ok);
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    response.set_mut(r#"
        <title>Tic Tac Toe</title>
        <form action="/game/?id=22&X=4&O=3" method="post" enctype='text/plain'>
            <input type="submit" name="make_move" value="Move"/>
        </form>
    "#);
    Ok(response)
}
fn make_board(req: &mut Request) -> IronResult<Response> {
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
    response.set_mut(status::Ok);
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    Ok(response)
}

fn end_game(request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();
    Ok(response)
}
