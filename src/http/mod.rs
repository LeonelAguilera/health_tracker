use std::{fs, io::{BufRead, BufReader, Read, Write}, net::TcpStream};
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct HttpPacket{
    pub query: String,
    pub _version: String,
    pub payload: Option<String>,
}

#[derive(Debug)]
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
        /*
        let mut s = buf_reader.lines();
        let s: Vec<String> = s.map(|result| result.unwrap()).take_while(|line| !line.is_empty()).collect();

        //Content-Length: 122
        */
    }
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

