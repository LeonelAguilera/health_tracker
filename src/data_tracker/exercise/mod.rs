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

    fn update(&mut self, db: &Connection){
        let (n_reps, weight): (Vec<_>, Vec<_>) = (1..(self.n_reps.len() as u8 + 1)).map(|serie| read_data_from_db(db, &self.name, serie)).unzip();

        self.n_reps = n_reps;
        self.weight = weight;
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
    pub fn new(db: &Connection) -> Result<Self, String>{
        let exercise_plan = fs::read_to_string(EXERCISE_PLAN_PATH).unwrap();
        let exercise_plan = json::parse(&exercise_plan).or(Err("Malformed JSON".to_string()))?;

        let semanario = match exercise_plan {
            JsonValue::Object(semanario) => semanario,
            f => return Err(format!("Malformed JSON\n\tOn first read\n\tExpected parent object to be Object\n\tInstead got: {f:#?}").to_string()),
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
        }).collect::<Result<Vec<Vec<ColeccionEjercicios>>, String>>()?
        .try_into()
            .ok()
            .ok_or("Malformed JSON:\n\tWeek len was not 7 days".to_string())
            .map(|contents| Self(contents))
    }

    pub fn get_todays_list(&self) -> Vec<Ejercicio> {
        let now = chrono::offset::Local::now();
        let weekday = now.weekday().num_days_from_monday() as usize;
        let current_day = now.ordinal() as usize;

        self.0[weekday].iter().map(|coleccion| match coleccion {
            ColeccionEjercicios::Single(ejercicio) => Vec::from([ejercicio.clone()]),
            ColeccionEjercicios::Switch(ejercicios) => {
                let switch_len = ejercicios.len();
                let mut ejercicios = ejercicios.clone();
                ejercicios.rotate_right(current_day % switch_len);
                ejercicios
            }
        }).flatten().collect()
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

