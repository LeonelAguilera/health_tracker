use std::{fs, time::{SystemTime, UNIX_EPOCH}};

use chrono::Datelike;
use json::{object::Object, JsonValue};
use rusqlite::Connection;

use crate::http::httperrors::HttpError;

const EXERCISE_PLAN_PATH: &str = "databases/exercise_plan.json";

#[derive(Debug)]
pub struct Ejercicio{
    name: String,
    n_reps: Vec<u8>,
    weight: Vec<u16>
}

impl Ejercicio {
    fn new(db: &Connection, origin: &Object) -> Result<Self, HttpError>{
        let name = origin.get("Name").ok_or(HttpError::InternalServerError("Malformed JSON:\n\tOn exercise creation\n\tExercise name does not exist"))?;
        let name = match name {
            JsonValue::Short(name) => name.to_string(),
            JsonValue::String(name) => name.to_string(),
            _ => return Err(HttpError::InternalServerError("Malformed JSON:\n\tOn exercise creation\n\tExpected String on exercise name")),
        };
        let series = origin.get("Series").ok_or(HttpError::InternalServerError("Malformed JSON:\n\tOn exercise creation\n\tExercise reps does not exist"))?;
        let series: f64 = match series {
            JsonValue::Number(series) => (*series).into(),
            _ => return Err(HttpError::InternalServerError("Malformed JSON:\n\tOn exercise creation\n\tExpected Number on exercise name")),
        };

        let (n_reps, weight): (Vec<_>, Vec<_>) = (1..(series as u8 + 1)).map(|serie| read_data_from_db(db, &name, serie)).unzip();

        Ok(Ejercicio{
            name,
            n_reps,
            weight,
        })
    }

    pub fn to_htmx_form(&self, current_exercise: usize, curr_set: usize) -> Result<Vec<u8>, HttpError>{
        if self.weight.len() != self.n_reps.len(){
            return Err(HttpError::InternalServerError("Data was stored malformed for this exercise"));
        }
        if curr_set >= self.weight.len(){
            return Err(HttpError::NotFound("This exercise does not have this many sets"));
        }

        let exercise_name = self.name.clone();
        let weight = self.weight[curr_set];
        let reps = self.n_reps[curr_set];

        let mut next_set = curr_set + 1;
        let mut next_exercise = current_exercise;
        if next_set == self.n_reps.len(){
            next_set = 0;
            next_exercise += 1;
        }
        let response = format!(include_str!("../../../html/templates/exercise_data_form.html"), exercise_name = exercise_name, current_set = (curr_set + 1), weight = weight, reps = reps, next_exercise = next_exercise, next_set = next_set);
        return Ok(response.as_bytes().to_vec());
    }

    fn update(&mut self, db: &Connection){
        let (n_reps, weight): (Vec<_>, Vec<_>) = (1..(self.n_reps.len() as u8 + 1)).map(|serie| read_data_from_db(db, &self.name, serie)).unzip();

        self.n_reps = n_reps;
        self.weight = weight;

        assert_eq!(self.n_reps.len(), self.weight.len());
    }
}

impl Clone for Ejercicio {
    fn clone(&self) -> Self {
        Self { name: self.name.clone(), n_reps: self.n_reps.clone(), weight: self.weight.clone()}
    }
}

enum ColeccionEjercicios {
    Single(Ejercicio),
    Switch(Vec<Ejercicio>),
}

impl ColeccionEjercicios {
    fn update(&mut self, db: &Connection){
        match self {
            Self::Single(ejercicio) => ejercicio.update(db),
            Self::Switch(ejercicios) => ejercicios.iter_mut().for_each(|ejercicio| ejercicio.update(db)),
        }
    }
}

pub struct PlanEjercicio([Vec<ColeccionEjercicios>; 7]);

impl PlanEjercicio {
    pub fn new(db: &Connection) -> Result<Self, HttpError>{
        let exercise_plan = fs::read_to_string(EXERCISE_PLAN_PATH).map_err(|_|HttpError::InternalServerError("Failed to load training plan"))?;
        let exercise_plan = json::parse(&exercise_plan).or(Err(HttpError::InternalServerError("Malformed training plan")))?;

        let semanario = match exercise_plan {
            JsonValue::Object(semanario) => semanario,
            _ => return Err(HttpError::InternalServerError("Malformed JSON\n\tOn first read\n\tExpected parent object to be Object")),
        };

        semanario.iter().map(|dia| {
            match dia {
                (_, JsonValue::Array(entradas)) => entradas.iter()
                    .map(|entrada|{
                        if let JsonValue::Object(tipo_ejercicio) = entrada{
                            if let Some(JsonValue::Object(ejercicio)) = tipo_ejercicio.get("Single"){
                                Ejercicio::new(db, ejercicio).map(|ejercicio| ColeccionEjercicios::Single(ejercicio))
                            }
                            else if let Some(JsonValue::Array(ejercicios)) = tipo_ejercicio.get("Switch"){
                                ejercicios.iter().map(|o_ejercicio|{
                                    match o_ejercicio {
                                        JsonValue::Object(ejercicio) => Ejercicio::new(db, ejercicio),
                                        _ => Err(HttpError::InternalServerError("Malformed JSON:\n\tOn Switch\n\tExpected JsonValue::Object inside Array")),
                                    }
                                }).collect::<Result<Vec<Ejercicio>,HttpError>>()
                                .map(|switch_content|ColeccionEjercicios::Switch(switch_content))
                            }
                            else{
                                Err(HttpError::InternalServerError("Malformed JSON:\n\tOn exercise kind selection\n\tExpected either Object Single or Array Switch"))
                            }
                        }
                        else{
                            Err(HttpError::InternalServerError("Malformed JSON:\n\tOn exercise kind reading\n\tExpected the input type to be an object"))
                        }
                    }).collect::<Result<Vec<ColeccionEjercicios>,HttpError>>(),
                _ => Err(HttpError::InternalServerError("Malformed JSON\n\tOn {f} plan reading\n\tExpected Array")),
            }
        }).collect::<Result<Vec<Vec<ColeccionEjercicios>>, HttpError>>()?
        .try_into()
            .ok()
            .ok_or(HttpError::InternalServerError("Malformed JSON:\n\tWeek len was not 7 days"))
            .map(|contents| Self(contents))
    }

    pub fn get_todays_list(&self) -> Vec<Ejercicio> {
        let now = chrono::offset::Local::now();
        let weekday = now.weekday().num_days_from_monday() as usize;
        let current_day = now.ordinal() as usize;

        self.0[weekday].iter().flat_map(|coleccion| match coleccion {
            ColeccionEjercicios::Single(ejercicio) => Vec::from([ejercicio.clone()]),
            ColeccionEjercicios::Switch(ejercicios) => {
                let switch_len = ejercicios.len();
                let mut ejercicios = ejercicios.clone();
                ejercicios.rotate_right(current_day % switch_len);
                ejercicios
            }
        }).collect()
    }

    pub fn update(&mut self, db: &Connection) {
        self.0.iter_mut().for_each(|dia| dia.iter_mut().for_each(|plan| plan.update(db)));
    }
}

pub fn build_graph_htmx_from_exercise_plan(plan: Vec<Ejercicio>) -> String{
    let mut htmx = String::from(r##"
                                <button hx-target="#form-container" hx-get="/new_training">Start training</button>
                                <div id="graphs">
                                "##);
    for ejercicio in plan{
        let nombre_ejercicio = ejercicio.name;

        if ejercicio.n_reps.len() <= 1{
            let representacion_grafico = format!(r##"
                                                 <div class="graph-container">
                                                 <h3>{nombre_ejercicio}</h3>
                                                 <img src="graph/exercise_data/repetitions/exercise_name/{nombre_ejercicio}/wset/1/simple/30/" alt="{nombre_ejercicio} graph" id="{nombre_ejercicio}-graph">
                                                 </div>
                                                 "##);
            htmx.push_str(representacion_grafico.as_str());
        }
        else{
            let sets = (0..ejercicio.n_reps.len()).map(|x| (x + 1).to_string()).collect::<Vec<_>>().join("/");
            let representacion_grafico = format!(r##"
                                                 <div class="graph-container">
                                                 <h3>{nombre_ejercicio}</h3>
                                                 <img src="graph/exercise_data/repetitions/exercise_name/{nombre_ejercicio}/multi/30/wset/{sets}" alt="{nombre_ejercicio} graph" id="{nombre_ejercicio}-graph">
                                                 </div>
                                                 "##);
            htmx.push_str(representacion_grafico.as_str());
        }
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

pub fn log_exercise(db: &Connection, plan: &mut PlanEjercicio, query: &str, payload: &str) -> Result<Vec<u8>, HttpError>{
    save_exercise_in_db(db, payload)?;

    plan.update(db);
    let plan = plan.get_todays_list();

    let mut params = query.split("/");
    let exercise = params.nth(2);
    let exercise = exercise.ok_or(HttpError::BadRequest("Next exercise index not included"))?.parse::<usize>().map_err(|_|HttpError::BadRequest("Invalid exercise index"))?;
    if exercise >= plan.len(){
        return Ok(Vec::new());
    }

    let set = params.next().ok_or(HttpError::BadRequest("Next set not included"))?.parse::<usize>().map_err(|_|HttpError::BadRequest("Invalid set index"))?;

    plan[exercise].to_htmx_form(exercise, set)
}

fn save_exercise_in_db(db: &Connection, payload: &str) -> Result<(), HttpError>{
    let mut exercise_name = None;
    let mut current_set = None;
    let mut weight = None;
    let mut repetitions = None;

    for element in payload.split("&"){
        let split_parameter = element.split("=").collect::<Vec<_>>();
        match split_parameter[0] {
            "exercise_name" => exercise_name = Some(split_parameter[1].to_owned()),
            "current_set" => current_set = Some(split_parameter[1].parse::<usize>()),
            "weight" => weight = Some(split_parameter[1].parse::<f64>()),
            "repetitions" => repetitions = Some(split_parameter[1].parse::<usize>()),
            _ => {},
        }
    }

    let exercise_name = exercise_name.ok_or(HttpError::BadRequest("Missing parameter \"exercise_name\""))?;
    let current_set = current_set.ok_or(HttpError::BadRequest("Missing parameter \"current_set\""))?.map_err(|_|HttpError::BadRequest("Bad format on \"series\" parameter"))?;
    let weight = weight.ok_or(HttpError::BadRequest("Missing parameter \"weight\""))?.map_err(|_|HttpError::BadRequest("Bad format on \"series\" parameter"))?;
    let repetitions = repetitions.ok_or(HttpError::BadRequest("Missing parameter \"repetitions\""))?.map_err(|_|HttpError::BadRequest("Bad format on \"repetitions\" parameter"))?;
    let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_|HttpError::InternalServerError("Server time previous to Unix Epoch"))?.as_secs();

    let _ = db.execute("INSERT INTO exercise_data VALUES (:tim, :exercise_name, :wset, :duration, :repetitions, :weight);", &[
                       (":tim", current_timestamp.to_string().as_str()),
                       (":exercise_name", exercise_name.as_str()),
                       (":wset", current_set.to_string().as_str()),
                       (":duration", 0_i64.to_string().as_str()),
                       (":repetitions", repetitions.to_string().as_str()),
                       (":weight", weight.to_string().as_str()),
    ]).map_err(|_|HttpError::InsuficientStorage("Could not save the new data"))?;

    return Ok(());
}
