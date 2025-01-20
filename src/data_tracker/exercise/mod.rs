use std::fs;

use chrono::Datelike;
use json::{object::Object, JsonValue};
use rusqlite::Connection;

const EXERCISE_PLAN_PATH: &str = "databases/exercise_plan.json";

#[derive(Debug)]
pub struct Ejercicio{
    name: String,
    n_reps: Vec<u8>,
    weight: Vec<u16>
}

impl Ejercicio {
    fn new(db: &Connection, origin: &Object) -> Result<Self, String>{
        let name = origin.get("Name").ok_or_else(|| "Malformed JSON:\n\tOn exercise creation\n\tExercise name does not exist".to_string())?;
        let name = match name {
            JsonValue::Short(name) => name.to_string(),
            JsonValue::String(name) => name.to_string(),
            f => return Err(format!("Malformed JSON:\n\tOn exercise creation\n\tExpected String on exercise name\n\tInstead got: {f:#?}").to_string()),
        };
        let series = origin.get("Series").ok_or_else(|| "Malformed JSON:\n\tOn exercise creation\n\tExercise reps does not exist".to_string())?;
        let series: f64 = match series {
            JsonValue::Number(series) => (*series).into(),
            f => return Err(format!("Malformed JSON:\n\tOn exercise creation\n\tExpected Number on exercise name\n\tInstead got: {f:#?}").to_string()),
        };

        let (n_reps, weight): (Vec<_>, Vec<_>) = (1..(series as u8 + 1)).map(|serie| read_data_from_db(db, &name, serie)).unzip();

        Ok(Ejercicio{
            name,
            n_reps,
            weight,
        })
    }
}

enum ColeccionEjercicios {
    Single(Ejercicio),
    Switch(Vec<Ejercicio>),
}

struct PlanEjercicio([Vec<ColeccionEjercicios>; 7]);

impl PlanEjercicio {
    fn new(db: &Connection) -> Result<Self, String>{
        let exercise_plan = fs::read_to_string(EXERCISE_PLAN_PATH).unwrap();
        let exercise_plan = json::parse(&exercise_plan).or(Err("Malformed JSON".to_string()))?;

        let semanario = match exercise_plan {
            JsonValue::Object(semanario) => semanario,
            f => return Err(format!("Malformed JSON\n\tOn first read\n\tExpected parent object to be Object\n\tInstead got: {f:#?}").to_string()),
        };

        let plan: Result<[Vec<ColeccionEjercicios>;7], _> = semanario.iter().map(|dia| {
            match dia {
                (_, JsonValue::Array(entradas)) => entradas.iter()
                                                      .map(|entrada|{
                                                          if let JsonValue::Object(tipo_ejercicio) = entrada{
                                                              if let Some(JsonValue::Object(ejercicio)) = tipo_ejercicio.get("Single"){
                                                                  match Ejercicio::new(db, ejercicio) {
                                                                      Ok(ejercicio) => Ok(ColeccionEjercicios::Single(ejercicio)),
                                                                      Err(err) => Err(err),
                                                                  }
                                                              }
                                                              else if let Some(JsonValue::Array(ejercicios)) = tipo_ejercicio.get("Switch"){
                                                                  ejercicios.iter().map(|o_ejercicio|{
                                                                      match o_ejercicio {
                                                                          JsonValue::Object(ejercicio) => Ejercicio::new(db, ejercicio),
                                                                          f => Err(format!("Malformed JSON:\n\tOn Switch\n\tExpected JsonValue::Object inside Array\n\tInstead found: {f:#?}").to_string()),
                                                                      }
                                                                  }).collect::<Result<Vec<Ejercicio>,String>>()
                                                                  .map(|switch_content|ColeccionEjercicios::Switch(switch_content))
                                                              }
                                                              else{
                                                                  Err(format!("Malformed JSON:\n\tOn exercise kind selection\n\tExpected either Object Single or Array Switch\n\tInstead got: {tipo_ejercicio:#?}").to_string())
                                                              }
                                                          }
                                                          else{
                                                              Err(format!("Malformed JSON:\n\tOn exercise kind reading\n\tExpected the input type to be an object\n\tInstead got: {entrada:#?}").to_string())
                                                          }
                                                      }).collect::<Result<Vec<ColeccionEjercicios>,String>>(),
                (f, _) => Err(format!("Malformed JSON\n\tOn {f} plan reading\n\tExpected Array\n\tInstead got: {f:#?}").to_string()),
            }
        }).collect::<Result<Vec<Vec<ColeccionEjercicios>>, String>>()?.try_into().ok().ok_or("Malformed JSON:\n\tWeek len was not 7 days".to_string());
        return plan;
    }
}


pub fn get_todays_list() -> Result<Vec<Ejercicio>, String>{
    let exercise_plan = fs::read_to_string(EXERCISE_PLAN_PATH).unwrap();
    let exercise_plan = json::parse(&exercise_plan).unwrap();
    let exercise_plan = json_to_list(exercise_plan);
    return exercise_plan;
}

fn json_to_list(input_json: JsonValue) -> Result<Vec<Ejercicio>, String>{
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
                          match Ejercicio::new(ejercicio) {
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
                                  JsonValue::Object(ejercicio) => Ejercicio::new(ejercicio),
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

pub fn build_graph_htmx_from_exercise_plan(plan: Vec<Ejercicio>) -> String{
    let mut htmx = String::from(r##"
                                <button hx-target="#form-container" hx-get="/new_training">Start training</button>
                                <div id="graphs">
                                "##);
    for ejercicio in plan{
        let nombre_ejercicio = ejercicio.name;
        let representacion_grafico = format!(r##"
                                             <div class="graph-container">
                                             <h3>{nombre_ejercicio}</h3>
                                             <img src="graph/{nombre_ejercicio}" alt="{nombre_ejercicio} graph" id="{nombre_ejercicio}-graph">
                                             </div>
                                             "##);
        htmx.push_str(representacion_grafico.as_str());
    }

    htmx.push_str("</div>");
    return htmx;
}
fn read_data_from_db(db: &Connection, exercise_name: &String, serie: u8) -> (u8, u16){
    let exercise_name = format!("'{exercise_name}'");
    let query = format!("SELECT repetitions, weight
                         FROM exercise_data
                         WHERE(
                             exercise_name = {exercise_name}
                             AND
                             wset = {serie}
                         )
                         ORDER BY timev DESC
                         LIMIT 1;
                         ");
    return db.query_row(&query, [], |row| Ok((row.get::<usize, u8>(0).unwrap_or(0), row.get::<usize, u16>(1).unwrap_or(0)))).unwrap_or((0, 0));
}

//Useless but don't want to lose it because is beautiful

fn json_to_exercise_list(db: &Connection, input_json: JsonValue) -> Result<Vec<Ejercicio>, String>{
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


