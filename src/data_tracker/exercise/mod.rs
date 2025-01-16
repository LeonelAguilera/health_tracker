use chrono::Datelike;

const EXERCISE_PLAN_PATH: &str = "databases/exercise_plan.json";

struct Ejercicio{
    name: String,
    n_reps: Vec<u8>,
}

pub fn get_todays_list(){
    let now = chrono::offset::Local::now();
    let weekday = now.weekday();
    let current_day = now.ordinal();
}
