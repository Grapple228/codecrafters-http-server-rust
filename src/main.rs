// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, BufWriter, Write}, 
    net::{TcpListener, TcpStream}, vec};

use itertools::Itertools;

fn split_request(reader: BufReader<&mut TcpStream>) -> Vec<String> {
    reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect()
}

fn handle_client(mut stream: TcpStream){
    // READ REQUEST
    let reader = BufReader::new(&mut stream);
    let request: Vec<_> = split_request(reader);
    
    println!("Incoming request: {request:#?}");

    let request_line = &request[0];

    let first_line_parts: Vec<&str> = request_line.split(" ").collect();
    
    let (response_status, response_message) = match first_line_parts[1] {
        "/" | "/index.html" => ("200", "OK"),
        _ => ("404", "Not Found")
    };

    let status_line = format!("HTTP/1.1 {response_status} {response_message}");

    let mut contents: String = request.into_iter().collect();
    contents = "".to_string();
    let length = contents.len();

    // WRITING RESPONSE
    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

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
