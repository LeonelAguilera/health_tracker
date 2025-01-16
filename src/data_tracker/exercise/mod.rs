use std::fs;

use chrono::Datelike;
use json::JsonValue;

const EXERCISE_PLAN_PATH: &str = "databases/exercise_plan.json";

#[derive(Debug)]
struct Ejercicio{
    name: String,
    n_reps: Vec<u8>,
}

type PlanEjercicio = Vec<Vec<Ejercicio>>;

pub fn get_todays_list(){
    let exercise_plan = fs::read_to_string(EXERCISE_PLAN_PATH).unwrap();
    let exercise_plan = json::parse(&exercise_plan).unwrap();
    let exercise_plan = json_to_list(exercise_plan);

    println!("{exercise_plan:#?}");
}

fn json_to_list(input_json: JsonValue) -> Result<Vec<Vec<Ejercicio>>, String>{
    let now = chrono::offset::Local::now();
    let weekday = now.weekday().to_string();
    let current_day = now.ordinal();

    match input_json {
        JsonValue::Object(dia) => {
            let plan = dia.get(&weekday).unwrap().to_owned();
            match plan {
                JsonValue::Array(entradas) =>{
                    for entrada in entradas{
                        match entrada {
                            JsonValue::Object(tipo_ejercicio) => {
                                tipo_ejercicio.iter().for_each(|elemento|
                                                               println!("{elemento:#?}\n####################################################################################\n")
                                                               );
                            },
                            _ => return Err("Malformed JSON".to_string()),
                        }
                    }
                    todo!();
                },
                _ => return Err("Malformed JSON".to_string()),
            }
        },
        _=> Err("Malformed JSON".to_string()),
    }
}
