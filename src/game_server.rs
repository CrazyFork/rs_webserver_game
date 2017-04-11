// Test with echo '{"user_id":"223344", "move_to":"4", "new_game":true }' > /dev/tcp/localhost/3001
extern crate rustc_serialize;
extern crate common;

use ::common::UserData;
use rustc_serialize::json;
use std::env;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex}; // for safely threading
use std::thread::spawn; // spawning threads
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};

fn main() {
    let mut args = env::args(); // args is an iter

    let address = match args.nth(1) {
        Some(s) => s,
        None => { println!("Usage: game_server <ip>:<port>"); return },
    };
    let listener = TcpListener::bind(address).unwrap();

    let tictac_data = Arc::new(TicTacGame::new());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // closure that calls a func to operate on the stream
                let tictac_child = tictac_data.clone();
                spawn(|| handle_client(stream, tictac_child));
            }
            Err(e) => { println!("Bad connection: {:?}", e); }
        }
    }
}

/// Game logic deals with this, and users board is parsed to JSON for sending
#[derive(RustcEncodable)]
struct TicTacBoard { board: HashMap<u32, Vec<Vec<char>>> }
struct TicTacGame { data: Mutex<TicTacBoard>,
}
impl TicTacGame {
    fn new() -> TicTacGame {
        TicTacGame { data: Mutex::new( TicTacBoard { board: HashMap::new() }), }
    }
    /// Inserts a new blank game for user_id
    fn new_game(&self, user_id: u32) {
        // move % columns = col (x)
        // move / columns = row (y)
        let array = vec!(vec!('0','1','2'), // row 0, x=0,1,2
                         vec!('3','4','5'), // row 1
                         vec!('6','7','8'));// row 2
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
    fn insert_move(&self, user_id: u32, place: i32, piece: char) -> Result<bool, String> {
        let mut guard = self.data.lock().unwrap(); // critical section begins
        let mut board = match guard.board.get_mut(&user_id) {
            Some(x) => x,
            None => return Err(format!("Game for user {:?} does not exist", user_id)),
        };
        let x = place % 3;
        let y = place / 3;
        board[y as usize][x as usize] = piece;
        return Ok(true)
    } 
}

fn write_error(mut stream: TcpStream, msg: String) {
    println!("{:?}", msg); // to console
    stream.write_all(msg.as_bytes()).unwrap(); // to webserver
}

/// Take stream and convert from JSON, perform logic, send JSON back
/// A new game can be started by recieving;
/// {"user_id":"number", "move_to":"-1", "new_game":true } 
fn handle_client(mut stream: TcpStream, game: Arc<TicTacGame>) {
    //stream.set_read_timeout(None).expect("set_read_timeout call failed");
    //stream.set_write_timeout(None).expect("set_write_timeout call failed");
    stream.set_ttl(100).expect("set_ttl call failed");

    // Read the incoming stream in to a buffer for working with
    let mut buffer = String::new();
    for byte in Read::by_ref(&mut stream).bytes() {
        let c = byte.unwrap() as char;
        buffer.push(c);
        if buffer.ends_with("\r\n\r\n") { break }
    }
    
    // decode the buffer from JSON to the UserData struct
    let user_data: UserData;
    match json::decode(&buffer) {
        Err(e) => { let msg = format!("Invalid JSON recieved: {:?}", e);
                    write_error(stream, msg); return
                  },
        Ok(o) => user_data = o,
    }
    
    if user_data.new_game {
        game.new_game(user_data.user_id)
    }
    if user_data.move_to >= 0 && user_data.move_to <= 8 {
        match game.insert_move(user_data.user_id, user_data.move_to, 'X') {
            Err(e) => { let msg = format!("User {:?} does not exist: {:?}", user_data.user_id, e);
                        write_error(stream, msg); return
                      },
            Ok(_) => {}, // ignore the success value, we only care about error
        }
    }
    
    println!("JSON = {:?}", game.get_json(user_data.user_id));
    match game.get_json(user_data.user_id) {
        Err(e) => { let msg = format!("Could not fetch JSON for user {:?}: {:?}", user_data.user_id, e);
                    write_error(stream, msg); return
                  },
        Ok(o) => stream.write_all(o.as_bytes()).unwrap(),
    }
}
