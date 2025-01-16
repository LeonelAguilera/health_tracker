use std::fs;

use chrono::Datelike;
use image::codecs::webp;
use json::JsonValue;
use rusqlite::Connection;

const EXERCISE_PLAN_PATH: &str = "databases/exercise_plan.json";

#[derive(Debug)]
struct Ejercicio{
    name: String,
    n_reps: Vec<u8>,
}

type PlanEjercicio = Vec<Vec<Ejercicio>>;

pub fn get_todays_list(db: &Connection){
    let exercise_plan = fs::read_to_string(EXERCISE_PLAN_PATH).unwrap();
    let exercise_plan = json::parse(&exercise_plan).unwrap();
    let exercise_plan = json_to_list(db, exercise_plan);

    println!("{exercise_plan:#?}");
}

fn json_to_list(db: &Connection, input_json: JsonValue) -> Result<Vec<Vec<Ejercicio>>, String>{
    let now = chrono::offset::Local::now();
    let weekday = now.weekday().to_string();
    let current_day = now.ordinal();

    let dia = match input_json {
        JsonValue::Object(dia) => dia,
        _ => return Err("Malformed JSON".to_string()),
    };

    let plan = dia.get(&weekday).ok_or_else(|| "Missing weekday plan".to_string())?;

    let entradas = match plan {
        JsonValue::Array(entradas) => entradas,
        _ => return Err("Malformed JSON".to_string()),
    };

    for entrada in entradas{
        let tipo_ejercicio = match entrada {
            JsonValue::Object(tipo_ejercicio) => tipo_ejercicio.iter(),
            _ => return Err("Malformed JSON".to_string()),
        };

        for elemento in tipo_ejercicio{
            match elemento {
                ("Single", JsonValue::Object(ejercicio)) => {
                    let name = ejercicio.get("Name").ok_or_else(|| "Malformed JSON".to_string())?;
                    let name = match name {
                        JsonValue::String(name) => name.to_string(),
                        _ => return Err("Malformed JSON".to_string()),
                    };
                    let reps = ejercicio.get("Reps").ok_or_else(|| "Malformed JSON".to_string())?;
                    let reps: f64 = match reps {
                        JsonValue::Number(reps) => (*reps).into(),
                        _ => return Err("Malformed JSON".to_string()),
                    };

                    let n_reps = (1..(reps as u8 + 1)).map(|serie| read_data_from_db(db, &name, serie)).collect::<Vec<u8>>();

                    let ejercicio = Ejercicio{
                        name,
                        n_reps,
                    };
                },
                ("Switch", JsonValue::Array(ejercicios)) => todo!(),
                _ => return Err("Malformed JSON".to_string()),
            }
        }
    }
    return Ok(vec![vec![Ejercicio{name: "asdf".to_string(), n_reps: vec![12,10,8]}]]);
}

fn read_data_from_db(db: &Connection, exercise_name: &String, serie: u8) -> u8{
    let query = format!("SELECT repetitions
                         FROM exercise_data
                         WHERE(
                             exercise_name = {exercise_name}
                             AND
                             wset = {serie}
                         )
                         ORDER BY timev DESC
                         LIMIT 1;
                         ");
    return db.query_row(&query, [], |row| row.get::<usize, u8>(0)).unwrap();
}
