// Uncomment this block to pass the first stage
use std::{
        collections::HashMap, default, env::{self, vars}, fmt, fs::File, io::{BufRead, BufReader, BufWriter, Read, Write}, net::{TcpListener, TcpStream}, path::{self, Path}};

use bytes::buf;
use http_server_starter_rust::ThreadPool;
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
    contents: Vec<u8>,
    content_type:String
}

impl Response {
    fn to_string(&self) -> String{
        let var_name: String = format!("HTTP/1.1 {0} {1}\r\nContent-Type: {2}\r\nContent-Length: {3}\r\n\r\n{4}", 
        self.status_code, 
        self.status_message, 
        self.content_type,
        self.contents.len(), 
        String::from_utf8(self.contents.to_owned()).unwrap());
        var_name
    }
}

fn process_request(request: Request) -> Response {    
    let path_segments: &[&str] = &split_into(&request.path, '/');
    
    let mut status_code = StatusCode::OK;
    let mut content_type: &str = "text/plain";
    let mut contents: Vec<u8> = "".as_bytes().to_owned();
    let mut status_message: &str = "OK";
    let _directory: String;

    match &path_segments {
        [] | ["index.html"] => {},
        ["user-agent"] => {
            contents = request.user_agent.as_bytes().to_owned();
        }
        ["echo", _echo] => {
            contents = _echo.as_bytes().to_owned();
        },
        ["files", _filename] => {
            _directory = env::var("directory").unwrap_or("Path not set".to_string());

            let combined_path = format!("{_directory}/{_filename}");
            let _path =Path::new(&combined_path);
            
            println!("Trying to get file '{_path:#?}'");

            if _path.exists(){
                let _file = File::open(_path).unwrap();

                let mut reader = BufReader::new(_file);
                let _ = reader.read_to_end(&mut contents);
                
                content_type = "application/octet-stream";
            } else{
                status_code = StatusCode::NotFound;
                status_message = "Not Found";
            }
        },
        _ => {
            status_code = StatusCode::NotFound;
            status_message = "Not Found";
        }
    };

    {
        let status_message = status_message.to_owned();
        let content_type = content_type.to_owned();
        Response { status_code, status_message, contents, content_type }
    }
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

fn process_agrs() {
    let args: Vec<String> = env::args().collect();
    println!("Processing args:\n{args:#?}");
    
    let mut hashmap: HashMap<String, String> = HashMap::new();

    let mut key: String = String::new();
    let mut is_has_key: bool = false;
    let mut iter = IntoIterator::into_iter(args);
    loop {
        match iter.next() {
            Some(i) => {
                let value = i.as_str();
                match &value[..2] {
                    "--" => {
                        is_has_key = true;
                        key = value.to_string().replace("--", "");
                    }
                    _ => {
                        if is_has_key{
                            hashmap.insert(key.to_string(), value.to_owned());
                            is_has_key = false;
                        }
                    }
                }
            },
            None => break
        }
    }
    
    for (key, value) in hashmap.into_iter() {
        env::set_var(key, value);
    }
}

fn main() {
    process_agrs();

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                pool.execute(||{
                    println!("accepted new connection");
                    handle_client(_stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
