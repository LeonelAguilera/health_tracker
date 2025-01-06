mod http;
mod data_tracker;

use std::{net::{TcpListener, TcpStream}, str::FromStr};

use data_tracker::scale::ScaleParameters;
use http::{not_found, simple_file_response, RequestType::{self, GET, POST}};
use rusqlite::Connection;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let db = open_database();

    for stream in listener.incoming(){
        let stream = stream.unwrap();

        println!("Conexión establecida");
        connection_handler(stream, &db);
    }
}

fn connection_handler(stream: TcpStream, db: &Connection){
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
            "/update" => {
                let body = ScaleParameters::from_str(&contents.payload.unwrap());
                println!("{body:#?}");
                db.execute("INSERT INTO scale_data VALUES (:ts, :)", params)
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

fn open_database() -> Connection {
    let con = Connection::open("./databases/data.db3").unwrap();

    let _ = con.execute("CREATE TABLE IF NOT EXISTS scale_data (
            time TIMESTAMP PRIMARY KEY,
            weight DECIMAL(3,2) NOT NULL,
            imc DECIMAL(3,2) NOT NULL,
            body_fat DECIMAL(3,2) NOT NULL,
            visceral_fat DECIMAL(3,2) NOT NULL,
            muscle DECIMAL(3,2) NOT NULL,
            water DECIMAL(3,2) NOT NULL,
            protein DECIMAL(3,2) NOT NULL,
            metabolism DECIMAL(3,2) NOT NULL,
            bone_mass DECIMAL(3,2) NOT NULL,
            hip_diameter DECIMAL(3,2) NOT NULL,
            )", ());

    return con;
}
