// Uncomment this block to pass the first stage
use std::{
        io::{BufRead, BufReader, BufWriter, Write}, 
        net::{TcpListener, TcpStream}};

fn split_request(reader: BufReader<&mut TcpStream>) -> Vec<String> {
    reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect()
}

fn get_response(segments: &[&str]) -> (String, String, String) {
    let _last: &str = &segments.last().unwrap_or(&"");
    
    let (response_status, status_message, response_contents) = match &segments {        
        [] | ["index.html"] => ("200", "OK", ""), 
        ["echo", _last] => ("200", "OK", *_last),
        _ => ("404", "Not Found", "")
    };

    (response_status.to_string(), status_message.to_string(), response_contents.to_string())
}

fn get_segments(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}

fn handle_client(mut stream: TcpStream){
    // READ REQUEST
    let reader = BufReader::new(&mut stream);
    let request: Vec<_> = split_request(reader);
    
    println!("Incoming request: {request:#?}");
    
    let request_line = &request[0];

    let first_line_parts: Vec<&str> = request_line.split(" ").collect();

    let segments: Vec<&str> = get_segments(&first_line_parts[1]);

    let (response_status, status_message, response_contents) = get_response(&segments); 

    let status_line = format!("HTTP/1.1 {response_status} {status_message}");
    let length = response_contents.len();

    // WRITING RESPONSE
    let response =
        format!("{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n{response_contents}");

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
