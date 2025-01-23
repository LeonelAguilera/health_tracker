mod httperrors;
use std::{fs, io::{BufReader, Read, Write}, net::TcpStream};

use httperrors::HttpError;
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///
pub const OK: &str = "HTTP/1.1 200 OK";

#[derive(Debug)]
pub struct HttpPacket{
    pub query: String,
    pub _version: String,
    pub payload: Option<String>,
}

#[derive(Debug)]
#[allow(unused)]
pub enum RequestType{
    OPTIONS(HttpPacket),
    GET(HttpPacket),
    POST(HttpPacket),
    PUT(HttpPacket),
    DELETE(HttpPacket),
    HEAD(HttpPacket),
    TRACE(HttpPacket),
    CONNECT(HttpPacket),
    PATCH(HttpPacket),
}
impl RequestType{
    pub fn new(stream: &TcpStream) -> Result<Self, ()> {
        let mut buf_reader = BufReader::new(stream);
        let mut buf = String::new();

        while buf.len() < 4{
            let mut caracter = [0; 1];
            let _ = buf_reader.read_exact(&mut caracter);
            buf.push(caracter[0] as char);
        }

        while buf[buf.len()-4..] != *"\r\n\r\n"{
            let mut caracter = [0; 1];
            let _ = buf_reader.read_exact(&mut caracter);
            buf.push(caracter[0] as char);
        }

        let mut header = buf.lines();
        let request_line = header.next().unwrap();

        let parts: Vec<&str> = request_line.split(" ").collect();
        if parts.len() != 3{
            return Err(());
        }
        let payload_size = header
            .find(|line| line.starts_with("Content-Length: "))
            .and_then(|linea| {
                match linea.split(": ").nth(1) {
                    Some(longitud) => Some(String::from(longitud)),
                    None => None,
                }
            })
            .and_then(|longitud| longitud.trim().parse::<usize>().ok())
            .unwrap_or(0);

        let inner = HttpPacket{
            query: parts[1].to_string(),
            _version: parts[2].to_string(),
            //raw: s.join("\n"),
            payload: {
                if payload_size > 0{
                    let mut buf = vec![0; payload_size];
                    let _ = buf_reader.read_exact(&mut buf);
                    let payload_str = String::from_utf8(buf).unwrap();

                    Some(payload_str)
                }
                else{
                    None
                }
            },
        };
        match parts[0] {
            "OPTIONS" => Ok(Self::OPTIONS(inner)),
            "GET" => Ok(Self::GET(inner)),
            "POST" => Ok(Self::POST(inner)),
            "PUT" => Ok(Self::PUT(inner)),
            "DELETE" => Ok(Self::DELETE(inner)),
            "HEAD" => Ok(Self::HEAD(inner)),
            "TRACE" => Ok(Self::TRACE(inner)),
            "CONNECT" => Ok(Self::CONNECT(inner)),
            "PATCH" => Ok(Self::PATCH(inner)),
            _ => Err(()),
        }
    }
}

pub fn simple_file_response(stream: TcpStream, path: &str){
    file_response(stream, OK, path);
}

pub fn not_found(stream: TcpStream){
    file_response(stream, "HTTP/1.1 404 NOT FOUND", "html/404.html");
}

pub fn file_response(stream: TcpStream, status_line: &str, path: &str){
    byte_stream_response(stream, status_line, Ok(fs::read_to_string(path).unwrap().into_bytes()));
}

pub fn byte_stream_response(mut stream: TcpStream, status_line: &str, byte_stream: Result<Vec<u8>, String>){
    let mut response;
    match byte_stream {
        Ok(mut byte_stream) => {
            response = format!("{status_line}\r\nContent-Length: {}\r\n\r\n", byte_stream.len()).into_bytes();
            response.append(&mut byte_stream);
        }
        Err(err)=> response = format!("HTTP/1.1 422 {err}").into_bytes(),
    }
    let _ = stream.write_all(&response);
}

pub fn empty_ok(mut stream: TcpStream){
    let _ = stream.write_all(OK.as_bytes());
}

pub fn send_error(mut stream: TcpStream, error: HttpError) {
    let status_line = format!("HTTP/1.1 {} {}", error.error_code(), error.message());
    let _ = stream.write_all(&status_line.into_bytes());
}
