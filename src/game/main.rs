extern crate rustc_serialize;

use rustc_serialize::json;
use std::env;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex}; // for safely threading
use std::thread::spawn; // spawning threads
use std::collections::HashMap;
use std::net::{TcpListener, SocketAddr, TcpStream};

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
    fn new_game(&mut self, user_id: u32) {
        // move % columns = col (x)
        // move / columns = row (y)
        let array = vec!(vec!('0','1','2'), // row 0, x=0,1,2
                         vec!('3','4','5'), // row 1
                         vec!('6','7','8'));// row 2
        let mut guard = self.data.lock().unwrap(); // critical section begins
        guard.board.insert(user_id, array); // guard is dropped automatically at end of scope
    }
    fn get_user_game(&mut self, user_id: u32) -> Vec<Vec<char>> {
        let guard = self.data.lock().unwrap();
        guard.board.get(&user_id).unwrap().to_vec()
    }
    fn insert_move(&mut self, user_id: u32, place: u32, piece: char) {
        let mut guard = self.data.get_mut().unwrap(); // critical section begins
        let mut board = guard.board.get_mut(&user_id).unwrap();
        let x = place % 3;
        let y = place / 3;
        board[y as usize][x as usize] = piece
    } 
}

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

// incoming data is parsed to this
#[derive(RustcDecodable, Debug)]
struct UserData {
    user_id: u32,
    move_to: u32,
    new_game: bool,
}

// Take stream and convert from JSON, perform logic, send JSON back
fn handle_client(mut stream: TcpStream, game: Arc<TicTacGame>) {
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
    let user_data: UserData;
    match json::decode(&buffer) {
        Err(e) => { println!("Invalid JSON recieved: {:?}", e); return },
        Ok(o) => user_data = o,
    }
    println!("Converted from JSON = {:?}", user_data);

    //stream.write_all(test).unwrap();
}
