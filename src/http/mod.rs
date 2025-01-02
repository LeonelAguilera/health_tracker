use std::{fs, io::{BufRead, BufReader, Write}, net::TcpStream, str::FromStr};

pub enum RequestType{
    OPTIONS,
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    TRACE,
    CONNECT,
    PATCH,
}

impl FromStr for RequestType{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OPTIONS" => Ok(Self::OPTIONS),
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "HEAD" => Ok(Self::HEAD),
            "TRACE" => Ok(Self::TRACE),
            "CONNECT" => Ok(Self::CONNECT),
            "PATCH" => Ok(Self::PATCH),
            _ => Err(()),
        }
    }
}

pub fn read_request(stream: &TcpStream) -> (RequestType, String){
    let buf_reader = BufReader::new(stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let parts: Vec<&str> = request_line.split(" ").collect();

    let request = RequestType::from_str(parts[0]);
    let query   = parts[1].to_string();

    return (request.unwrap(), query);
}

pub fn simple_file_response(stream: TcpStream, path: &str){
    file_response(stream, "HTTP/1.1 200 OK", path);
}

pub fn not_found(stream: TcpStream){
    file_response(stream, "HTTP/1.1 404 NOT FOUND", "html/404.html");
}

pub fn file_response(mut stream: TcpStream, status_line: &str, path: &str){
    let contents = fs::read_to_string(path).unwrap();
    let len = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {len}\r\n\r\n{contents}");
    let _ = stream.write_all(response.as_bytes());
}
