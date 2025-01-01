use std::{fs, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming(){
        let stream = stream.unwrap();

        println!("Conexión establecida");
        connection_handler(stream);
    }
}

fn connection_handler(mut stream: TcpStream){
    let buf_reader = BufReader::new(&stream);

    let http_request: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    println!("Request: {http_request:#?}");

    simple_file_response(stream, "html/index.html");
}

fn simple_file_response(mut stream: TcpStream, path: &str){
    let status_ine = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string(path).unwrap();
    let len = contents.len();

    let response = format!("{status_ine}\r\nContent-Length: {len}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
