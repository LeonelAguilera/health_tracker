mod image_wrapper;

use chrono::{Datelike, TimeZone};
use rusqlite::{Connection, Error};
use image_wrapper::Imagen;

const ALLOWED_COLUMN_NAMES: &[&str] = &["weight", "imc", "body_fat", "visceral_fat", "muscle", "water", "protein", "metabolism", "bone_mass", "hip_diameter"];
const ALLOWED_TABLES_NAMES: &[&str] = &["scale_data"];


pub fn graph_maker(db: &Connection, column_name: &str, table_name: &str, res_x: usize, res_y: usize) -> Result<Vec<u8>, String>{
    let data = read_data_from_db(db, column_name, table_name).unwrap();

    let mut graph = Imagen::new_empty_graph(res_x, res_y);
    graph.draw_horizontal_lines(&data);
    graph.draw_line(&data);
    
    return Ok(graph.into_bytes());
}

fn read_data_from_db(db: &Connection, column_name: &str, table_name: &str) -> Result<Vec<(i64, f64)>, String>{
    if !ALLOWED_COLUMN_NAMES.contains(&column_name){
        return Err("Columna no válida".to_string());
    }
    if !ALLOWED_TABLES_NAMES.contains(&table_name){
        return Err("Tabla no válida".to_string());
    }

    let current_time = chrono::offset::Local::now();
    let eod_timestamp = chrono::offset::Local.with_ymd_and_hms(current_time.year(), current_time.month(), current_time.day(), 23, 59, 59).unwrap().timestamp();
    let cutoff_timestamp = eod_timestamp - ((image_wrapper::GRAPH_NUM_DAYS * 24 * 3600) as i64);

    let query = format!("SELECT timev, {}
                        FROM {}
                        WHERE timev >= {}
                        UNION
                        SELECT timev, {}
                        FROM(
                            SELECT timev, {}
                            FROM {}
                            WHERE timev <  {}
                            ORDER BY timev DESC
                            LIMIT 1
                        );", column_name, table_name, cutoff_timestamp, column_name, column_name, table_name, cutoff_timestamp);
    let mut statement = db.prepare(&query).unwrap();

    return Ok(statement.query_map([], |row|{
        let date = row.get::<usize, i64>(0);
        let value = row.get::<usize, f64>(1);

        match (date, value) {
            (Ok(date), Ok(value)) => Ok((date, value)),
            _ => Err(Error::InvalidQuery),
        }
    }).unwrap()
    .filter_map(|x| x.ok())
    .collect::<Vec<(i64, f64)>>());
}
