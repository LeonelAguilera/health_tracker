mod http;

use std::{io::{BufRead, BufReader}, net::{TcpListener, TcpStream}};

use http::{not_found, read_request, simple_file_response, Request};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming(){
        let stream = stream.unwrap();

        println!("Conexión establecida");
        connection_handler(stream);
    }
}

fn connection_handler(stream: TcpStream){
    let request = read_request(&stream);
    match request {
        (Request::GET, path) if path == "/" => simple_file_response(stream, "html/index.html"),
        _ => not_found(stream),
    }
}
