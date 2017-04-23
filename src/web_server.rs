extern crate rustc_serialize;
extern crate common;

use common::{Request, Response, UserData, Status};
use rustc_serialize::json;
use std::env;
use std::io::{Read, Write};
use std::fs::File;
use std::thread::spawn; // spawning threads
use std::net::{TcpListener, TcpStream, Shutdown};

/// Helper function for reading files, will return a 500 Status Response
/// which can be modified or sent to the client
///
fn read_file(name: &str) -> Result<Vec<u8>, Response> {
    let mut file = match File::open(name) {
        Ok(o) => o,
        Err(e) => {
            let mut res = Status::internal_error();
            res.body(format!("{:?} not found: {:?}", name, e).into_bytes());
            return Err(res);
        }
    };
    let mut buf = Vec::new();
    match file.read_to_end(&mut buf) {
        Ok(_) => {}
        Err(e) => {
            let mut res = Status::internal_error();
            res.body(format!("Error reading {:?}: {:?}", name, e).into_bytes());
            return Err(res);
        }
    }
    Ok(buf)
}

fn main() {
    let mut args = env::args(); // args is an iter
    // Using .nth() discards all preceding elements in the iterator
    // since it is a consuming iterator. Some(ref s) would prevent this.
    let web_address = match args.nth(1) {
        Some(s) => s,
        None => {
            println!("Started on localhost:3000");
            "localhost:3000".to_string()
        }
    };

    // Start the listener on address provided
    let listener = TcpListener::bind(web_address).unwrap();
    // I originally was spawning a base thread that contained this loop
    // which would prevent blocking. But for this assignment it isn't really
    // required, and doing without makes the code a little cleaner.
    // Not: The for loop for `.incoming()` is infinite.
    for stream in listener.incoming().by_ref() {
        match stream {
            Ok(mut stream) => {
                // For each incoming stream we spawn a thread using a closure.
                // The keyword `move` shifts the `stream` in to the thread, i.e
                // it takes ownership of the stream (connection).
                spawn( move || {
                    // Hand off a mutable reference to `parse_stream`, here we are
                    // using the analogue of a C pointer.
                    // & = reference, or "borrow" in Rust parlance
                    let request = match Request::parse_stream(&mut stream) {
                        // Pattern match the Result returned, this helps us prevent crashes,
                        // panics, poisoning threads etc.
                        Ok(request) => request,
                        // If an error is encountered in the parsing we just print and
                        // return from the spawned thread.
                        Err(e) => {
                            println!("Parsing stream to a request failed: {:?}", e);
                            return;
                        }
                    };
                    let response: Response;
                    // The Request is parsed and jammed in to a Request data struct
                    // A url can also contain a file name, eg index.html which can be
                    // parsed easily anywhere desired, such as "/game/index.html".
                    // I had initially created a hashmap of callbacks for this, which works
                    // well for the task, but in the interest of being "to the point" didn't
                    // use it.
                    match request.url.as_ref() {
                        // A basic router style match
                        "/" =>      response = handle_new(&request),
                        "/game/" => response = handle_tictac(&request),
                        _ =>        response = Status::not_found(),
                    }
                    
                    // For these two if statements we are only interested in whether or not
                    // they were an Err Result. The content of the error could be extracted
                    // using a `match` if desired.
                    if stream.write_all(response.to_string().as_bytes()).is_err() {
                        println!("Write to connection failed");
                        return;
                    }
                    if stream.shutdown(Shutdown::Both).is_err() {
                        println!("Could not shutdown stream correctly: prematurely closed?");
                        return;
                    }
                });
            }
            // Lastly, the initial connection attempt may have failed, so print and continue
            Err(e) => println!("Bad connection: {:?}", e),
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
        None => "123",// TODO - fetch unused id from game server
    };
    // Create a response from a template, this returns a prebuilt Response struct
    let mut response = Status::ok();

    // As we saw at the top of the source, read_file() will return a prebuilt
    // Response depending on its own results. We can just return this directly
    // to the thread to use in -- "/" =>      response = handle_new(&request),
    let index_file = match read_file("index.html") {
        Ok(o) => o,
        Err(e) => return e,
    };

    // Replace {user_id} as the vec<u8> is parsed to a String
    // For extra safety, we do pattern matching and return a prebuilt
    // Response if we ever encounter an error.
    let body_work = match String::from_utf8(index_file) {
        Ok(s) => s.replace("{user_id}", user_id),
        // the `_` here is a "wildcard" match, it basically throws away the
        // value contained in Err()
        Err(_) => return Status::internal_error(),
    };

    // The body of the response stores a Vec<u8> type, so we need to convert
    // the string to a vector array of u8 bytes.
    response.body(body_work.into_bytes());
    // And lastly, insert an extra header with the length of the body
    let body_len = &response.body_len().to_string();
    response.header("Content-Length", body_len);
    response
}

/// The main handler for client games.
///
/// This function fetches values from the parsed request and
/// serializes them to JSON for transmission to the game_server.
/// Upon recieving a response it then deserializes, and parses
/// to an html table string for insertion in to the html string.
///
fn handle_tictac(request: &Request) -> Response {
    // Request body is optional, need to check it exists first
    // the .get_param() return type is Result<&String, Response>
    // We get either an Ok(&String) or an Err(Response)
    // Of note here is we get a reference to a string, or we get a
    // Response `moved` to here, i.e, take ownership of that data,
    // and ownership moves upwards with each return
    let user_id = match request.get_param("user_id") {
        Ok(user) => user,
        Err(e) => return e,
    };
    let move_to = match request.get_param("move_to") {
        Ok(mv) => mv.as_bytes()[0] as char,
        Err(_) => 'n',
    };
    // We can get around this typing stuff by using an enum storage for the body hashmap
    // ... Maybe later
    let new_game = match request.get_param("new_game") {
        Ok(ng) => {
            match ng.as_str() {
                "false" => false,
                "true" => true,
                _ => false,
            }
        }
        Err(_) => true, // Maybe shouldn't ignore the error, but the other fields are fine
    };

    // Create the filled struct using the above variables
    let user_data = UserData {
        user_id: user_id.parse::<u32>().unwrap(),
        move_to: move_to,
        new_game: new_game,
    };
    // Create the JSON string to send to the game server
    let user_json = match json::encode(&user_data) {
        Err(e) => format!("JSON conversion failed: {:?}", e),
        Ok(string) => {
            // This was an attempt to get the game server to accept codes, and it does work,
            // it just wasn't suitable here. Instead we're returning a Response with a message.
            // This could easily be crafted to show the current game + a message somewhere
            // on the page
            let s = match move_to {
                '0' => "0:".to_string() + &string,
                '1' => "0:".to_string() + &string,
                '2' => "0:".to_string() + &string,
                '3' => "0:".to_string() + &string,
                '4' => "0:".to_string() + &string,
                '5' => "0:".to_string() + &string,
                '6' => "0:".to_string() + &string,
                '7' => "0:".to_string() + &string,
                '8' => "0:".to_string() + &string,
                _ => { //"1:".to_string() + &string,
                    let mut response = Status::ok();
                    response.body("Illegal move, please press back".as_bytes().to_vec());
                    let body_len = &response.body_len().to_string();
                    response.header("Content-Length", body_len);
                    return response;
                }
            };
            s
        }
    };
    println!("JSON = {:?}", user_json);

    // Send JSON to game_server and parse received JSON to data structure (vec)
    // using the helper function
    let game = match rw_user_data(&user_json, "localhost:3001") {
        Ok(game) => {
            // NOTE: Any place with a `.unwrap()` is a potential crash, this should be replaced
            // with the right handling such as the `match` statements seen so far.
            // The `.unwrap` just yanks the Ok/Some variable out and panics! if Err/None
            // it /can/ be okay iff we know for sure it is safe.
            match json::decode(&String::from_utf8(game).unwrap()) {
                Err(_) => return Status::internal_error(),
                Ok(o) => o,
            }
        }
        Err(_) => return Status::internal_error(),
    };

    // Remember the read_file helper function returns a Response to use if Err()
    let game_file = match read_file("game.html") {
        Ok(o) => o,
        Err(e) => return e,
    };

    // Create the html table using the helper function
    let game_table = create_table(game);

    // Start crafting a new response using the ok() preset
    let mut response = Status::ok();
    // Chain the `.replace`, this string function is a copy iterator over the string, which
    // replaces any pattern encountered as it does so
    let body_work = String::from_utf8(game_file)
        .unwrap() // potentially a crash spot
        .replace("{user_id}", user_id)
        .replace("{game_table}", &game_table);
    // Insert our new body in to the response. The body is
    // a Vec<u8> so transform the string in to a vector of bytes.
    response.body(body_work.into_bytes()); 

    let body_len = &response.body_len().to_string();
    response.header("Content-Length", body_len);
    response
}

/// Helper function to write to the game_server and listen to output
///
/// The return type is a Result - Result<Vec<u8>, String>
///
fn rw_user_data(user_data: &str, addr: &str) -> Result<Vec<u8>, String> {
    let mut game: Vec<u8> = Vec::new();
    // Pattern matching to try and safely handle all possible results.
    match TcpStream::connect(addr) {
        Ok(ref mut o) => {
            if o.write_all(user_data.as_bytes()).is_err() {
                return Err(format!("Could not write to {:?}", addr));
            }
            if o.shutdown(Shutdown::Write).is_err() {
                return Err(format!("Stream to {:?} ended prematurely?", addr));
            }
            if o.read_to_end(&mut game).is_err() {
                return Err(format!("Could not read from {:?}", addr));
            }
            return Ok(game);
        }
        Err(e) => return Err(format!("Game server down? {:?}", e)),
    };
}

/// A simple iterator over the game data to produce an HTML table
///
fn create_table(game: Vec<Vec<char>>) -> String {
    let mut game_table = String::new();
    for row in game {
        game_table.push_str("<tr>");
        for col in row {
            game_table.push_str("<td>[");
            game_table.push(col);
            game_table.push_str("]</td>");
        }
        game_table.push_str("</tr>");
    }
    game_table
}
