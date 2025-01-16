mod http;
mod data_tracker;
mod graph_maker;

use std::{net::{TcpListener, TcpStream}, str::FromStr, time::{SystemTime, UNIX_EPOCH}};

use data_tracker::scale::ScaleParameters;
use graph_maker::graph_maker;
use http::{byte_stream_response, empty_ok, not_found, simple_file_response, RequestType::{self, GET, POST}, OK};
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
            "/train_data" => {data_tracker::exercise::get_todays_list(); not_found(stream)},
            "/new_data_form" => simple_file_response(stream, "html/templates/new_data_form.html"),
            path if path.starts_with("/styles/") => simple_file_response(stream, &path[1..]),
            path if path.starts_with("/graph/")  => byte_stream_response(stream, OK, graph_maker(db, &path[7..], "scale_data", 1080, 1080)),
            _ => {
                not_found(stream)
            },
        },
        POST(contents) => match  contents.query.as_str() {
            "/update" => {ScaleParameters::from_str(&contents.payload.unwrap()).unwrap().save_to_db(db); empty_ok(stream);},
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

    if let Err(err) = con.execute("CREATE TABLE IF NOT EXISTS scale_data (
            timev INTEGER UNSIGNED PRIMARY KEY,
            weight DECIMAL(4,2) NOT NULL,
            imc DECIMAL(4,2) NOT NULL,
            body_fat DECIMAL(4,2) NOT NULL,
            visceral_fat DECIMAL(4,2) NOT NULL,
            muscle DECIMAL(4,2) NOT NULL,
            water DECIMAL(4,2) NOT NULL,
            protein DECIMAL(4,2) NOT NULL,
            metabolism DECIMAL(6,2) NOT NULL,
            bone_mass DECIMAL(4,2) NOT NULL,
            hip_diameter DECIMAL(4,2) NOT NULL
            );", ()){
        println!("Database scale_data table creation failed: {err}");
    }
    if let Err(err) = con.execute("CREATE TABLE IF NOT EXISTS exercise_data(
            timev INTEGER UNSIGNED PRIMARY KEY,
            exercise_name VARCHAR(50),
            wset TINYINT UNSIGNED,
            duration SMALLINT UNSIGNED,
            repetitions TINYINT UNSIGNED
            );", ()){
        println!("Database exercise_data table creation failed: {err}");
    }

    //insert_dummy_data(&con);

    return con;
}

#[allow(dead_code)]
fn insert_dummy_data(con:  &Connection){
    let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let half_day_diff = 12*3600;

    for i in 0..28{
        if let Err(err) = con.execute("INSERT INTO scale_data VALUES (:tim, :a, :b, :c, :d, :e, :f, :g, :h, :i, :j);", &[
                                      (":tim", (current_timestamp - (i * half_day_diff)).to_string().as_str()),
                                      (":a", (70.0 + (i as f64)).to_string().as_str()),
                                      (":b", (70.1 + (i as f64)).to_string().as_str()),
                                      (":c", (70.2 + (i as f64)).to_string().as_str()),
                                      (":d", (70.3 + (i as f64)).to_string().as_str()),
                                      (":e", (70.4 + (i as f64)).to_string().as_str()),
                                      (":f", (70.5 + (i as f64)).to_string().as_str()),
                                      (":g", (70.6 + (i as f64)).to_string().as_str()),
                                      (":h", (70.7 + (i as f64)).to_string().as_str()),
                                      (":i", (70.8 + (i as f64)).to_string().as_str()),
                                      (":j", (70.9 + (i as f64)).to_string().as_str()),
        ]){
            println!("Dummy data insertion in scale_data failed in {i}: {err}");
        }
    }

    let example_exercises = vec!["Bicep_Curl", "Hammer_Curl", "Concentration_Curl"];
    for i in 0..28{
        for exercise_name in &example_exercises{
            for rep in 1..4{
                if let Err(err) = con.execute("INSERT INTO exercise_data VALUES (:tim, :a, :b, :c, :d);", &[
                                              (":tim", (current_timestamp - (i * half_day_diff) + (rep * (exercise_name.len() as u64))).to_string().as_str()),
                                              (":a", exercise_name),
                                              (":b", (rep).to_string().as_str()),
                                              (":c", (60+2*i).to_string().as_str()),
                                              (":d", (8+((i+rep)%6)).to_string().as_str()),
                ]){
                    println!("Dummy data insertion in scale_data failed in {i}, {exercise_name}@{rep}: {err}");
                }
            }
        }
    }
}

