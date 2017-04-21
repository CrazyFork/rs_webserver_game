// Test with echo '{"user_id":"223344", "move_to":"4", "new_game":true }' > /dev/tcp/localhost/3001
extern crate rustc_serialize;
extern crate common;

use common::UserData;
use rustc_serialize::json;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex}; // for safely threading
use std::thread::spawn; // spawning threads
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, Shutdown};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3001").unwrap();

    let tictac_data = Arc::new(TicTacGame::new());

    for stream in listener.incoming().by_ref() {
        match stream {
            Ok(mut stream) => {
                // closure that calls a func to operate on the stream
                let tictac_child = tictac_data.clone();
                spawn(move || { handle_client(&mut stream, tictac_child); });
            }
            Err(e) => println!("Bad connection: {:?}", e),
        }
    }
}

/// Game logic deals with this, and users board is parsed to JSON for sending
#[derive(RustcEncodable)]
struct TicTacBoard {
    board: HashMap<u32, Vec<Vec<char>>>,
}
struct TicTacGame {
    data: Mutex<TicTacBoard>,
}
impl TicTacGame {
    fn new() -> TicTacGame {
        TicTacGame { data: Mutex::new(TicTacBoard { board: HashMap::new() }) }
    }
    /// Inserts a new blank game for user_id
    fn new_game(&self, user_id: u32) {
        // move % columns = col (x)
        // move / columns = row (y)
        let array = vec![vec!['0', '1', '2'], // row 0, x=0,1,2
                         vec!['3', '4', '5'], // row 1
                         vec!['6', '7', '8']]; // row 2
        let mut guard = self.data.lock().unwrap(); // critical section begins
        guard.board.insert(user_id, array); // guard is dropped automatically at end of scope
    }
    fn get_json(&self, user_id: u32) -> Result<String, String> {
        let guard = self.data.lock().unwrap();
        let data = guard.board.get(&user_id).unwrap().to_vec();
        match json::encode(&data) {
            Err(e) => return Err(format!("JSON conversion failed: {:?}", e)),
            Ok(o) => Ok(o),
        }
    }
    fn insert_move(&self, user_id: u32, place: char, piece: char) -> Result<bool, String> {
        let mut guard = self.data.lock().unwrap(); // critical section begins
        let mut board = match guard.board.get_mut(&user_id) {
            Some(x) => x,
            None => return Err(format!("Game for user {:?} does not exist", user_id)),
        };
        let p = place.to_string().parse::<u32>().unwrap();
        if p > 8 {
            return Err(String::from("Illegal move"));
        }
        let x = p as i32 % 3;
        let y = p as i32 / 3;
        let pos = board[y as usize][x as usize];
        if pos == place {
            board[y as usize][x as usize] = piece;
            return Ok(true);
        } else {
            return Err(String::from("Illegal move"));
        }
    }
}

fn write_error(stream: &mut TcpStream, msg: String) {
    println!("{:?}", msg); // to console
    stream
        .write_all(msg.as_bytes())
        .unwrap_or(println!("Failed to write to stream")); // to webserver
    stream
        .shutdown(Shutdown::Write)
        .unwrap_or(println!("Stream closed before write?"));
}

/// Take stream and convert from JSON, perform logic, send JSON back
/// A new game can be started by receiving;
/// {"user_id":"number", "move_to":"-1", "new_game":true }
fn handle_client(stream: &mut TcpStream, game: Arc<TicTacGame>) {
    //stream.set_read_timeout(None).expect("set_read_timeout call failed");
    //stream.set_write_timeout(None).expect("set_write_timeout call failed");
    stream.set_ttl(100).expect("set_ttl call failed");

    // Read the incoming stream in to a buffer for working with
    // TODO read to buffer and save length of read - Do it differennt
    let mut buffer = String::new();
    for byte in Read::by_ref(stream).bytes() {
        buffer.push(byte.unwrap() as char);
    }

    let split: Vec<&str> = buffer.splitn(2, ':').collect();
    if split.len() < 2 {
        write_error(stream, format!("Invalid JSON received"));
        return;
    }
    let code = split[0].parse::<u32>().unwrap();
    let buffer = split[1];

    // decode the buffer from JSON to the UserData struct
    let user_data: UserData;
    match json::decode(&buffer) {
        Err(e) => {
            write_error(stream, format!("Invalid JSON received: {:?}", e));
            return;
        }
        Ok(o) => user_data = o,
    }

    if user_data.new_game {
        game.new_game(user_data.user_id)
    }

    // Return just the JSON without making a move - move can be anything
    if code == 1 {
        match game.get_json(user_data.user_id) {
            Err(e) => {
                let msg = format!("User {:?}: {:?}", user_data.user_id, e);
                write_error(stream, msg);
                return;
            }
            Ok(o) => {
                stream.write_all(o.as_bytes()).unwrap();
                stream.shutdown(Shutdown::Write).unwrap();
                return;
            }
        }
    }

    // Insert user move
    match game.insert_move(user_data.user_id, user_data.move_to, 'X') {
        Ok(_) => {}
        Err(_) => {
            // Return early if an error, write a valid JSON if possible
            match game.get_json(user_data.user_id) {
                Err(e) => {
                    let msg = format!("User {:?}: {:?}", user_data.user_id, e);
                    write_error(stream, msg);
                    return;
                }
                Ok(o) => {
                    stream.write_all(o.as_bytes()).unwrap();
                    stream.shutdown(Shutdown::Write).unwrap();
                    return;
                }
            }
        }
    }
    // Insert computer move
    for cpu in 0..8 {
        let ch = format!("{}", cpu).as_bytes()[0] as char;
        match game.insert_move(user_data.user_id, ch, 'O') {
            Ok(_) => break,
            Err(_) => {}
        }
    }

    println!("JSON = {:?}", game.get_json(user_data.user_id).unwrap());
    match game.get_json(user_data.user_id) {
        Err(e) => {
            let msg = format!("User {:?}: {:?}", user_data.user_id, e);
            write_error(stream, msg);
            return;
        }
        Ok(o) => {
            stream.write_all(o.as_bytes()).unwrap();
            stream.shutdown(Shutdown::Write).unwrap();
        }
    }
}
