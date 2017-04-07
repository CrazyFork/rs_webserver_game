use std::str::FromStr;
use std::io::Read;

fn main() {
    let mut args = env::args(); // args is an iter

    let web_address = match args.nth(1) {
        Some(s) => s,
        None => { println!("Usage: web_server <web_ip>:<port> <game_ip>:<port>"); return },
    };
    let game_address = match args.nth(2) {
        Some(s) => s,
        None => { println!("Usage: web_server <web_ip>:<port> <game_ip>:<port>"); return },
    };
    let listener = TcpListener::bind(web_address).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // closure that calls a func to operate on the stream
                // for each client stream, we will need to read the stream, parse to
                // json, and connect to the game server to do updates and get game board
                spawn(|| handle_client(stream));
            }
            Err(e) => { println!("Bad connection: {:?}", e); }
        }
    }
}

fn handle_client(mut stream: TcpStream, mut game: Arc<TicTacGame>) {
    //stream.set_read_timeout(None).expect("set_read_timeout call failed");
    //stream.set_write_timeout(None).expect("set_write_timeout call failed");
    stream.set_ttl(100).expect("set_ttl call failed");
    
    let mut buffer = String::new();
    for byte in Read::by_ref(&mut stream).bytes() {
        let c = byte.unwrap() as char;
        buffer.push(c);
        if buffer.ends_with("\r\n\r\n") { break }
    }
    for line in buffer.lines() { println!("{:?}", line)}
}
