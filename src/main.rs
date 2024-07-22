// Uncomment this block to pass the first stage
use std::{
        collections::HashMap, env::{self}, fmt, fs::{self, File}, io::{BufRead, BufReader, BufWriter, Read, Write}, net::{TcpListener, TcpStream}, path::Path};

use bytes::buf::{self, Reader};
use http_server_starter_rust::ThreadPool;
use itertools::Itertools;
enum RequestType {
    GET,
    POST,
    PUT,
    DELETE,
    UNKNOWN
}

#[derive(Debug, Copy, Clone)]
enum CompressionType{
    none,
    gzip
}

#[derive(Debug, Copy, Clone)]
enum StatusCode{
    Ok= 200,
    Created= 201,
    Bad= 400,
    Unauthorized= 401,
    Forbidden= 403,
    NotFound= 404,    
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self as i32)
    }
}

impl fmt::Display for CompressionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
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
    content_length: i32,
    contents: Vec<u8>,
    compression_type: CompressionType
}

fn split_into(path: &str, splitter: char) -> Vec<&str> {
    path.split(splitter).filter(|s| !s.is_empty()).collect()
}

impl Request {
    fn split_request(reader: &mut BufReader<&mut TcpStream>) -> Vec<String> {
        reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect()
    }

    /// Creates a new [`Request`].
    fn new(stream: &mut TcpStream) -> Self{
        let mut reader = BufReader::new(stream);

        let splitted_request =  Self::split_request(&mut reader);
        
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
        let mut contents: Vec<u8> = Vec::new();
        let mut compression_type = CompressionType::none;

        for line in &splitted_request{
            let (property, value) = line.split_once(": ").unwrap_or(("",""));

            match property {
                "Content-Type" => content_type = value.to_owned(),
                "User-Agent" => user_agent = value.to_owned(),
                "Accept" => accept = value.to_owned(),
                "Host" => host = value.to_owned(),
                "Content-Length" => content_length = value.parse().unwrap_or(0),
                "Accept-Encoding" => {
                    let allowed_compressions = value.split(", ").filter(|&x| !x.is_empty()).collect_vec();
                    if allowed_compressions.contains(&"gzip"){
                        compression_type = CompressionType::gzip;
                    }
                },
                _ => {}
            }
        };

        if content_length > 0{
            let mut buffer = vec![0; content_length as usize]; 
            reader.read_exact(&mut buffer).unwrap();
            contents = buffer
        }

        let request = Self{
            accept,
            request_type,
            path,
            protocol,
            user_agent,
            host,
            content_length,
            content_type,
            contents,
            compression_type
        };
        request
    }
}

struct Response{
    status_code: StatusCode,
    status_message: String,
    contents: Vec<u8>,
    content_type:String,
    content_encoding: CompressionType
}

impl Response {
    fn to_string(&self) -> String{
        let var_name: String = format!("HTTP/1.1 {0} {1}\r\nContent-Type: {2}\r\n{3}Content-Length: {4}\r\n\r\n{5}", 
        self.status_code, 
        self.status_message, 
        self.content_type,
        match &self.content_encoding{
            CompressionType::none => "".to_string(),
            ct => format!("Content-Encoding: {ct}\r\n"),
        },
        self.contents.len(), 
        String::from_utf8(self.contents.to_owned()).unwrap());
        var_name
    }

    fn not_found() -> Response{
        Response { status_code: StatusCode::NotFound, 
            status_message: "Not Found".to_string(), 
            contents: Vec::new(), 
            content_type: "text/plain".to_string(),
            content_encoding: CompressionType::none
            }
    }
}

fn process_request(request: Request) -> Response {
    let path_segments: &[&str] = &split_into(&request.path, '/');
    
    let mut status_message: &str = "OK";
    let mut status_code = StatusCode::Ok;
    let mut content_type: &str = "text/plain";
    let mut contents: Vec<u8> = "".as_bytes().to_owned();
    let _directory: String;

    match request.request_type{
        RequestType::GET =>{
            match &path_segments {
                [] | ["index.html"] => {},
                ["user-agent"] => { contents = request.user_agent.as_bytes().to_owned(); }
                ["echo", _echo] => { contents = _echo.as_bytes().to_owned(); },
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
                    } else {
                        return Response::not_found();
                    }
                },
                _ => { return Response::not_found(); }
            };
        },
        RequestType::POST => {
            match &path_segments {
                ["files", _filename] => {
                    _directory = env::var("directory").unwrap_or("Path not set".to_string());
        
                    let combined_path = format!("{_directory}/{_filename}");
                    let _path =Path::new(&combined_path);
                    
                    println!("Trying to write file '{_path:#?}'");

                    let file = fs::OpenOptions::new().create(true).append(true).open(&_path).unwrap();
                    
                    let mut file = BufWriter::new(file);
                    let _ = file.write(&request.contents);
                    let _ = file.flush();

                    status_message = "Created";
                    status_code = StatusCode::Created;
                },
                _ => { return Response::not_found(); }
            };
        },
        RequestType::PUT => {
            match &path_segments {
                _ => { return Response::not_found(); }
            };
        },
        RequestType::DELETE => {
            match &path_segments {
                _ => { return Response::not_found(); }
            };
        },
        _ => { return Response::not_found(); }
    }

    {
        let status_message = status_message.to_owned();
        let content_type = content_type.to_owned();
        Response { status_code, status_message, contents, content_type, content_encoding: request.compression_type }
    }
}

fn handle_client(mut stream: TcpStream){
    // READ REQUEST    
    let request = Request::new(&mut stream);

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
