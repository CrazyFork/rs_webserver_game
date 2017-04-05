extern crate iron;
extern crate router;
extern crate urlencoded;
#[macro_use] extern crate mime;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::str::FromStr;
use urlencoded::UrlEncodedBody;

fn main() {
    let mut router = Router::new();
    router.get("/", start_game);
    router.post("/:cpu/:user", post_move);

    println!("Serving from localhost:3000");
    Iron::new(router).http("localhost:3000").unwrap();
}

#[allow(unused_variables)]
fn start_game(request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();

    response.set_mut(status::Ok);
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    response.set_mut(r#"
        <title>Tic Tac Toe</title>
        <form action="/cpu/user" method="post">
            <input type="submit" name="start_game" value="Start Game"/>
        </form>
    "#);
    Ok(response)
}

fn post_move(req: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();

    let input_map;
    match req.get_ref::<UrlEncodedBody>() {
        Err(e) => {
            response.set_mut(status::BadRequest);
            response.set_mut(format!("Error parsing form data: {:?}\n", e));
            return Ok(response);
        }
        Ok(map) => { input_map = map; }
    }

    let content = format!(r#"
        <title>Tic Tac Toe</title>
        <form action="/cpu:{}/user:{}" method="post">
            <input type="text" name="place"/>
            <button type="submit">Make move</button>
        </form>
    "#, 4, 5);

    //
    let user_move;
    match input_map.get("place") {
        None => {
            response.set_mut(status::Ok);
            response.set_mut(mime!(Text/Html; Charset=Utf8));
            response.set_mut(content);
            return Ok(response);
        }
        Some(_move) => { user_move = _move}
    }

    response.set_mut(status::Ok);
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    response.set_mut(content);
    Ok(response)
}

fn end_game(request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();
    Ok(response)
}
