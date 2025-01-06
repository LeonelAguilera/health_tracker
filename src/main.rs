mod http;
mod data_tracker;

use std::{io::{BufReader, Read}, net::{TcpListener, TcpStream}, str::FromStr};

use data_tracker::scale::ScaleParameters;
use http::{not_found, simple_file_response, RequestType::{self, GET, POST}};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming(){
        let stream = stream.unwrap();

        println!("Conexión establecida");
        connection_handler(stream);
    }
}

fn connection_handler(stream: TcpStream){
    let request = RequestType::new(&stream).unwrap();
    println!("{request:#?}");
    match request {
        GET(contents) => match  contents.query.as_str() {
            "/" => simple_file_response(stream, "html/index.html"),
            "/scale_data" => simple_file_response(stream, "html/templates/health_data.html"),
            "/new_data_form" => simple_file_response(stream, "html/templates/new_data_form.html"),
            path if path.starts_with("/styles/") => simple_file_response(stream, &path[1..]),
            _ => {
                not_found(stream)
            },
        },
        POST(contents) => match  contents.query.as_str() {
            "/update" => {let body = ScaleParameters::from_str(&contents.payload.unwrap());
                println!("{body:#?}");
            }
            _ => {
                not_found(stream)
            },
        },
        _ => {
            not_found(stream)},
    }
    println!("Served\n\n");
}
