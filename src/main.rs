// Uncomment this block to pass the first stage
use std::{
        fmt, io::{BufRead, BufReader, BufWriter, Write}, net::{TcpListener, TcpStream}};

enum RequestType {
    GET,
    POST,
    PUT,
    DELETE,
    UNKNOWN
}


#[derive(Debug, Copy, Clone)]
enum StatusCode{
    OK= 200,
    Bad= 400,
    Unauthorized= 401,
    NotFound= 404,    
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self as i32)
    }
}

struct Request{
    request_type: RequestType,
    content_type: String,
    path: String,
    protocol: String,
    user_agent: String,
    host: String,
    accept: String,
    content_length: i32
}

fn split_into(path: &str, splitter: char) -> Vec<&str> {
    path.split(splitter).filter(|s| !s.is_empty()).collect()
}

impl Request {
    fn split_request(reader: BufReader<&mut TcpStream>) -> Vec<String> {
        reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect()
    }

    /// Creates a new [`Request`].
    fn new(reader: BufReader<&mut TcpStream>) -> Self{
        let splitted_request =  Self::split_request(reader);
        
        println!("Processing request:\n{splitted_request:#?}");

        let request_type: RequestType;
        let path: String;
        let protocol: String;

        let splitted_first = split_into(&splitted_request[0], ' ');

        request_type = match splitted_first[0].to_uppercase().as_str() {
            "GET" => RequestType::GET,
            "PUT" => RequestType::PUT,
            "POST" => RequestType::POST,
            "DELETE" =>  RequestType::DELETE,
            _ => RequestType::UNKNOWN
        };
        path = splitted_first[1].to_owned();
        protocol = splitted_first[2].to_owned();

        let mut content_type: String = String::new();
        let mut user_agent: String = String::new();
        let mut accept: String = String::new();
        let mut host: String = String::new();
        let mut content_length: i32 = 0;

        for line in &splitted_request{
            let (property, value) = line.split_once(": ").unwrap_or(("",""));

            match property {
                "Content-Type" => content_type = value.to_owned(),
                "User-Agent" => user_agent = value.to_owned(),
                "Accept" => accept = value.to_owned(),
                "Host" => host = value.to_owned(),
                "Content-Length" => content_length = value.parse().unwrap_or(0),
                _ => {}
            }
        };

        let request = Self{
            accept,
            request_type,
            path,
            protocol,
            user_agent,
            host,
            content_length,
            content_type,
        };
        request
    }
}

struct Response{
    status_code: StatusCode,
    status_message: String,
    contents: String
}

impl Response {
    fn new(status_code: StatusCode, status_message: String, contents: String) -> Self {
        Self { status_code, status_message, contents }
    }

    fn to_string(&self) -> String{
        let var_name: String = format!("HTTP/1.1 {0} {1}\r\nContent-Type: text/plain\r\nContent-Length: {2}\r\n\r\n{3}", 
        self.status_code, 
        self.status_message, 
        self.contents.len(), 
        self.contents);
        var_name
    }
}

fn process_request(request: Request) -> Response {    
    let path_segments: &[&str] = &split_into(&request.path, '/');
    
    let (status_code, status_message, contents): (StatusCode, &str, &str) = match &path_segments {
        [] | ["index.html"] => (StatusCode::OK, "OK", ""), 
        ["user-agent"] => (StatusCode::OK, "OK", &request.user_agent),
        ["echo", _echo] => (StatusCode::OK, "OK", *_echo),
        _ => (StatusCode::NotFound, "Not Found", "")
    };

    Response::new(status_code, status_message.to_owned(), contents.to_owned())
}

fn handle_client(mut stream: TcpStream){
    // READ REQUEST
    let reader = BufReader::new(&mut stream);
    
    let request = Request::new(reader);

    let response = process_request(request);

    let response_string = response.to_string();
    println!("Outcoming response:\n[\n{response_string}\n]");

    // WRITING RESPONSE
    let mut writer = BufWriter::new(&mut stream);
    if writer.write_all(response_string.as_bytes()).is_err(){
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
