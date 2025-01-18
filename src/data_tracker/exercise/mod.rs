use std::fs;

use chrono::Datelike;
use json::{object::Object, JsonValue};
use rusqlite::Connection;

const EXERCISE_PLAN_PATH: &str = "databases/exercise_plan.json";

#[derive(Debug)]
struct Ejercicio{
    name: String,
    n_reps: Vec<u8>,
}

impl Ejercicio {
    fn new(db: &Connection, origin: &Object) -> Result<Self, String>{
        let name = origin.get("Name").ok_or_else(|| "Malformed JSON:\n\tOn exercise creation\n\tExercise name does not exist".to_string())?;
        let name = match name {
            JsonValue::Short(name) => name.to_string(),
            JsonValue::String(name) => name.to_string(),
            f => return Err(format!("Malformed JSON:\n\tOn exercise creation\n\tExpected String on exercise name\n\tInstead got: {f:#?}").to_string()),
        };
        let reps = origin.get("Reps").ok_or_else(|| "Malformed JSON:\n\tOn exercise creation\n\tExercise reps does not exist".to_string())?;
        let reps: f64 = match reps {
            JsonValue::Number(reps) => (*reps).into(),
            f => return Err(format!("Malformed JSON:\n\tOn exercise creation\n\tExpected Number on exercise name\n\tInstead got: {f:#?}").to_string()),
        };

        let n_reps = (1..(reps as u8 + 1)).map(|serie| read_data_from_db(db, &name, serie)).collect::<Vec<u8>>();

        Ok(Ejercicio{
            name,
            n_reps,
        })
    }
}

pub fn get_todays_list(db: &Connection){
    let exercise_plan = fs::read_to_string(EXERCISE_PLAN_PATH).unwrap();
    let exercise_plan = json::parse(&exercise_plan).unwrap();
    let exercise_plan = json_to_list(db, exercise_plan);

    println!("{exercise_plan:#?}");
}

fn json_to_list(db: &Connection, input_json: JsonValue) -> Result<Vec<Ejercicio>, String>{
    let now = chrono::offset::Local::now();
    let weekday = now.weekday().to_string();
    let current_day = now.ordinal();

    let dia = match input_json {
        JsonValue::Object(dia) => dia,
        f => return Err(format!("Malformed JSON\n\tOn first read\n\tExpected parent object to be Object\n\tInstead got: {f:#?}").to_string()),
    };

    let plan = dia.get(&weekday).ok_or_else(|| format!("Missing weekday plan for {weekday}").to_string())?;

    let entradas = match plan {
        JsonValue::Array(entradas) => entradas,
        f => return Err(format!("Malformed JSON\n\tOn {weekday} plan reading\n\tExpected Array\n\tInstead got: {f:#?}").to_string()),
    };

    return Ok(entradas.iter()
        .map(|entrada|{
            if let JsonValue::Object(tipo_ejercicio) = entrada{
                if let Some(JsonValue::Object(ejercicio)) = tipo_ejercicio.get("Single"){
                    match Ejercicio::new(db, ejercicio) {
                        Ok(ejercicio) => Ok(vec![ejercicio]),
                        Err(err) => Err(err),
                    }
                }
                else if let Some(JsonValue::Array(ejercicios)) = tipo_ejercicio.get("Switch"){
                    let mut ejercicios = ejercicios.clone();
                    let displacement = (current_day as usize) % ejercicios.len();
                    ejercicios.rotate_right(displacement);
                    ejercicios.iter().map(|o_ejercicio|{
                        match o_ejercicio {
                            JsonValue::Object(ejercicio) => Ejercicio::new(db, ejercicio),
                            f => Err(format!("Malformed JSON:\n\tOn Switch\n\tExpected JsonValue::Object inside Array\n\tInstead found: {f:#?}").to_string()),
                        }
                    }).collect::<Result<Vec<Ejercicio>,String>>()
                }
                else{
                    Err(format!("Malformed JSON:\n\tOn exercise kind selection\n\tExpected either Object Single or Array Switch\n\tInstead got: {tipo_ejercicio:#?}").to_string())
                }
            }
            else{
                Err(format!("Malformed JSON:\n\tOn exercise kind reading\n\tExpected the input type to be an object\n\tInstead got: {entrada:#?}").to_string())
            }
        }).collect::<Result<Vec<Vec<Ejercicio>>,String>>()?.into_iter().flatten().collect::<Vec<Ejercicio>>());
}

fn read_data_from_db(db: &Connection, exercise_name: &String, serie: u8) -> u8{
    let exercise_name = format!("'{exercise_name}'");
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
    println!("YIPPIEEE");
    return db.query_row(&query, [], |row| row.get::<usize, u8>(0)).unwrap();
}
