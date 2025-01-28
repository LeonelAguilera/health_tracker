#![deny(clippy::unwrap_used)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_closure)]
//Internal modules
mod http;
mod data_tracker;
mod graph_maker;
use data_tracker::{exercise::{build_graph_htmx_from_exercise_plan, log_exercise, PlanEjercicio}, scale::ScaleParameters};
use graph_maker::graph_handler;
use http::{byte_stream_response, empty_ok, httperrors::HttpError, not_found, send_error, simple_file_response, RequestType::{self, GET, POST}, OK};

use core::panic;
//Standard library
use std::{net::{TcpListener, TcpStream}, str::FromStr, time::{SystemTime, UNIX_EPOCH}};

//Third party libraries
use rusqlite::Connection;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind TcpListener to provided address");
    let db = open_database();
    let mut exercise_plan = PlanEjercicio::new(&db).expect("Could not read exercise plan");

    for stream in listener.incoming(){
        let stream = match stream {
            Ok(stream) => stream,
            Err(_) => continue,
        };

        println!("Conexión establecida");
        connection_handler(stream, &db, &mut exercise_plan);
    }
}

fn connection_handler(stream: TcpStream, db: &Connection, exercise_plan: &mut PlanEjercicio){
    let request = match RequestType::new(&stream){
        Ok(request) => request,
        Err(_) => {send_error(&stream, HttpError::InternalServerError("Could not read request")); return;},
    };
    println!("{request:#?}");
    match request {
        GET(contents) => match contents.query.as_str() {
            "/" => simple_file_response(stream, "html/index.html"),
            "/scale_data" => simple_file_response(stream, "html/templates/health_data.html"),
            "/train_data" => {
                let today_plan = exercise_plan.get_todays_list();
                byte_stream_response(&stream, OK, Ok(build_graph_htmx_from_exercise_plan(today_plan).as_bytes().to_vec()))
            },
            "/new_data_form" => simple_file_response(stream, "html/templates/new_data_form.html"),
            "/new_training" => {
                let today_plan = exercise_plan.get_todays_list();
                byte_stream_response(&stream, OK, today_plan[0].to_htmx_form(0, 0));
            },
            query if query.starts_with("/styles/") => simple_file_response(stream, &query[1..]),
            query if query.starts_with("/graph/")  => {println!("Serving graph"); byte_stream_response(&stream, OK, graph_handler(db, query))},
            _ => {
                not_found(stream)
            },
        },
        POST(contents) => match  contents.query.as_str() {
            "/update_scale" => contents.payload
                .ok_or(HttpError::BadRequest("No payload found"))
                .and_then(|payload| String::from_utf8(payload).map_err(|_| HttpError::BadRequest("Impossible to parse payload")))
                .and_then(|payload_str| ScaleParameters::from_str(&payload_str))
                .and_then(|scale| scale.save_to_db(db))
                .map_or_else(|err| send_error(&stream, err), |_| empty_ok(&stream)),
            query if query.starts_with("/submit_exercise/") => contents.payload
                .ok_or(HttpError::BadRequest("No payload found"))
                .and_then(|payload| String::from_utf8(payload).map_err(|_|HttpError::BadRequest("Impossible to parse payload")))
                .map_or_else(|err| send_error(&stream, err), |payload_str| byte_stream_response(&stream, OK, log_exercise(db, exercise_plan, query, &payload_str))),
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
    let con = Connection::open("./databases/data.db3").expect("Could not open database");

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
        panic!();
    }
    if let Err(err) = con.execute("CREATE TABLE IF NOT EXISTS exercise_data(
            timev INTEGER UNSIGNED PRIMARY KEY,
            exercise_name VARCHAR(50),
            wset TINYINT UNSIGNED,
            duration SMALLINT UNSIGNED,
            repetitions TINYINT UNSIGNED,
            weight SMALLINT UNSIGNED
            );", ()){
        println!("Database exercise_data table creation failed: {err}");
        panic!();
    }

    //insert_dummy_data(&con);

    return con;
}

#[allow(dead_code)]
fn insert_dummy_data(con:  &Connection){
    let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Today is earlier than the Unix Epoch (at least in current system time)").as_secs();
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
                if let Err(err) = con.execute("INSERT INTO exercise_data VALUES (:tim, :a, :b, :c, :d, :e);", &[
                                              (":tim", (current_timestamp - (i * half_day_diff) + (rep * (exercise_name.len() as u64))).to_string().as_str()),
                                              (":a", exercise_name),
                                              (":b", (rep).to_string().as_str()),
                                              (":c", (60+2*i).to_string().as_str()),
                                              (":d", (8+((i+rep)%6)).to_string().as_str()),
                                              (":e", 10.to_string().as_str()),
                ]){
                    println!("Dummy data insertion in scale_data failed in {i}, {exercise_name}@{rep}: {err}");
                }
            }
        }
    }
}

