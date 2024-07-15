// Uncomment this block to pass the first stage
use std::{io::{BufWriter, Write}, net::{TcpListener, TcpStream}};

fn handle_client(mut stream: TcpStream){
    let response = "HTTP/1.1 200 OK\r\n\r\n";

    let mut writer = BufWriter::new(&mut stream);
    if writer.write_all(response.as_bytes()).is_err(){
        println!("Failed to response to stream!");
    }
    if writer.flush().is_err(){
        println!("Failed to flush stream!");
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                handle_client(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
