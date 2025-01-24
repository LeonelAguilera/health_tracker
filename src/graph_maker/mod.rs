mod image_wrapper;

use chrono::{Datelike, TimeZone};
use rusqlite::{params_from_iter, Connection, Error};
use image_wrapper::Imagen;

use crate::http::httperrors::HttpError;

const ALLOWED_COLUMN_NAMES: &[&str] = &["weight", "imc", "body_fat", "visceral_fat", "muscle", "water", "protein", "metabolism", "bone_mass", "hip_diameter"];
const ALLOWED_GRAPH_TYPES: &[&str] = &["single", "multi"];

#[derive(Debug)]
enum GraphType {
    Simple,
    Multi((String, Vec<String>)),
}

impl GraphType {
    fn new(kind: &str, query: Vec<String>) -> Result<Self, HttpError>{
        match kind {
            "single" => Ok(Self::Simple),
            "multi" => if query.len() > 2{
                Ok(Self::Multi((query[0].clone(), query[1..].to_vec())))
            }
            else{
                Err(HttpError::BadRequest("Not enough parameters for Multi graph"))
            },
            _ => Err(HttpError::BadRequest("Unknown graph type")),
        }
    }

    fn fetch_data_from_db(&self, db: &Connection, table_name: &str, column_name: &str, mut query_condition: String, parameters: Vec<String>, cutoff_timestamp: i64) -> Result<Vec<Vec<(i64, f64)>>, HttpError>{
        match self {
            Self::Simple => {
                let query = format!("SELECT timev, {column_name}
                                    FROM {table_name}
                                    WHERE timev >= {cutoff_timestamp}{query_condition}
                                    UNION
                                    SELECT timev, {column_name}
                                    FROM(
                                        SELECT timev, {column_name}
                                        FROM {table_name}
                                        WHERE timev <  {cutoff_timestamp}{query_condition}
                                        ORDER BY timev DESC
                                        LIMIT 1
                                    );");
                println!("\n\nQuery:\n{query}\n\n");
                let mut statement = db.prepare(&query).unwrap();
                return statement.query_map(params_from_iter(parameters.iter()), |row| {
                        let date = row.get::<usize, i64>(0);
                        let value = row.get::<usize, f64>(1);

                        if let (Ok(date), Ok(value)) = (date, value){
                            Ok((date, value))
                        }
                        else{
                            Err(Error::InvalidQuery)
                        }
                    })
                    .map_err(|_| HttpError::InternalServerError("Could not get the requested data"))?
                    .collect::<Result<Vec<(i64, f64)>,_>>()
                    .map_err(|_| HttpError::InternalServerError("Could not get all the data"))
                    .map(|ok| vec![ok]);
            }
            Self::Multi(selector) => {
                query_condition.push_str(format!("\nAND {} = ?{}", &selector.0, parameters.len() + 1).as_str());
                let query = format!("SELECT timev, {column_name}
                                    FROM {table_name}
                                    WHERE timev >= {cutoff_timestamp}{query_condition}
                                    UNION
                                    SELECT timev, {column_name}
                                    FROM(
                                        SELECT timev, {column_name}
                                        FROM {table_name}
                                        WHERE timev <  {cutoff_timestamp}{query_condition}
                                        ORDER BY timev DESC
                                        LIMIT 1
                                    );");
                println!("\n\nQuery:\n{query}\n\n");
                return selector.1.iter()
                    .map(|selector_data|{
                        let mut parameters = parameters.clone();
                        parameters.push(selector_data.to_owned());
                        let mut statement = db.prepare(&query).unwrap();
                        let return_val = statement.query_map(params_from_iter(parameters.iter()), |row| {
                            let date = row.get::<usize, i64>(0);
                            let value = row.get::<usize, f64>(1);
                            if let (Ok(date), Ok(value)) = (date, value){
                                Ok((date, value))
                            }
                            else{
                                Err(Error::InvalidQuery)
                            }
                        })
                        .map_err(|_| HttpError::InternalServerError("Could not get the requested data"))?
                        .collect::<Result<Vec<(i64, f64)>,_>>()
                        .map_err(|_| HttpError::InternalServerError("Could not get all the data"));
                        return_val
                    }
                ).collect::<Result<Vec<Vec<(i64, f64)>>,HttpError>>();
            }
        }
        /*
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
           */
    }
}

impl PartialEq for GraphType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Simple, Self::Simple) => true,
            (Self::Multi(lhs), Self::Multi(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

#[derive(Debug)]
struct GraphData{
    source_table: String,
    source_column: String,
    static_db_data: Vec<(String, String)>,
    gtype: GraphType,
    timescale: usize,
}

impl GraphData {
    fn new(query: &str) -> Result<Self, HttpError> {
        /*Graph format description:
         * /graph/table_name/graphing_column/column_1/value_1/column_2/value_2/.../column_n/value_n/graph_type/timescale/graph_type_dependant_parameters
         * graph_type can either be:
         *  Simple: Draws a line for a single parameter. Does not have extra parameters
         *  Multi: Draws multiple lines in a single graph depending on different values of a column. Extra partameters: column_name/value_1/value_2/.../value_k
         * timescale is the amount of time for wich is represented the data in days
         */
        let mut parts = query.split("/");
        _ = parts.next();
        if parts.next() != Some("graph") {
            return Err(HttpError::BadRequest("Not a graph request"));
        }
        let source_table = parts.next().ok_or(HttpError::BadRequest("No table to graph"))?.to_string();

        let source_column = parts.next().ok_or(HttpError::BadRequest("No column to graph"))?.to_string();

        let mut lhs = parts.next().ok_or(HttpError::BadRequest("No graph type selected"))?;
        let mut static_db_data = Vec::new();

        while ALLOWED_GRAPH_TYPES.contains(&lhs) == false {
            let rhs = parts.next().ok_or(HttpError::BadRequest("Column without value"))?;
            static_db_data.push((lhs.to_string(), rhs.to_string()));
            lhs = parts.next().ok_or(HttpError::BadRequest("No graph type selected"))?;
        }
        let graph_type = lhs;
        let timescale = parts.next().ok_or(HttpError::BadRequest("No timescale included"))?.parse::<usize>().ok().ok_or(HttpError::BadRequest("Bad timescale format"))? * 3600 * 24;
        let gtype = GraphType::new(graph_type, parts.map(|part| part.to_string()).collect())?;

        return Ok(Self{
            source_table,
            source_column,
            static_db_data,
            gtype,
            timescale,
        });
    }

    fn fetch_data_from_db(&self, db: &Connection) -> Result<Vec<Vec<(i64, f64)>>, HttpError>{
        let query = "SELECT * FROM sqlite_master WHERE type='table'";
        let mut statement = db.prepare(query).unwrap();

        let allowed_tables_names = statement.query_map([], |row| row.get::<usize, String>(1)).unwrap().collect::<Result<Vec<String>, _>>().map_err(|_| HttpError::InternalServerError("Unable of reading table names"))?;
        if !allowed_tables_names.contains(&self.source_table){
            return Err(HttpError::BadRequest("Invalid table name"));
        }

        let query = format!("PRAGMA table_info({})", self.source_table);
        statement = db.prepare(&query).unwrap();
        let allowed_column_names = statement.query_map([], |row| row.get::<usize, String>(1)).unwrap().collect::<Result<Vec<String>, _>>().map_err(|_| HttpError::InternalServerError("Unable of reading column names"))?;
        if !allowed_column_names.contains(&self.source_column){
            return Err(HttpError::BadRequest("Invalid column name"));
        }
        let mut query_condition = String::new();
        let mut parameters = Vec::new();
        for i in 0..self.static_db_data.len(){
            if !allowed_column_names.contains(&self.static_db_data[i].0) {
                return Err(HttpError::BadRequest("Invalid column name in static data"));
            }
            let condition = format!("\nAND {} = ?{}", &self.static_db_data[i].0, i+1);
            query_condition.push_str(condition.as_str());
            parameters.push(self.static_db_data[i].1.to_owned());
        }

        let current_time = chrono::offset::Local::now();
        let eod_timestamp = chrono::offset::Local.with_ymd_and_hms(current_time.year(), current_time.month(), current_time.day(), 23, 59, 59).unwrap().timestamp();
        let cutoff_timestamp = eod_timestamp - (self.timescale as i64);

        self.gtype.fetch_data_from_db(db, &self.source_table, &self.source_column, query_condition, parameters, cutoff_timestamp)
    }
}

pub fn graph_handler(db: &Connection, query: &str) -> Result<Vec<u8>, HttpError>{
    println!("Handling graph with query {query}");
    let graph = GraphData::new(query)?;
    println!("Data extracted:\n{graph:#?}");
    let data = graph.fetch_data_from_db(db);
    println!("{data:#?}");
    let image = Imagen::new_empty_graph(1080, 1080, graph.timescale, 1.0, 2.0);

    return Ok(image.into_bytes());
}

/*
pub fn scale_graph_maker(db: &Connection, column_name: &str, table_name: &str, res_x: usize, res_y: usize) -> Result<Vec<u8>, String>{
    let data = read_data_from_db(db, column_name, table_name).unwrap();
    if data.len() == 0{
        return Err("NO DATA TO DISPLAY".to_string());
    }

    let mut graph = Imagen::new_empty_graph(res_x, res_y, 7);
    graph.draw_horizontal_lines(&data);
    graph.draw_line(&data);
    
    return Ok(graph.into_bytes());
}*/

fn read_data_from_db(db: &Connection, column_name: &str, table_name: &str) -> Result<Vec<(i64, f64)>, String>{
    if !ALLOWED_COLUMN_NAMES.contains(&column_name){
        return Err("Columna no válida".to_string());
    }
    /*
    if !ALLOWED_TABLES_NAMES.contains(&table_name){
        return Err("Tabla no válida".to_string());
    }
    */

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

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn graph_query_parser_1(){
        let query = "/graph/scale_data/weight/single/7";
        let graph = GraphData::new(query).unwrap();
        assert_eq!(graph.source_table, "scale_data".to_string());
        assert_eq!(graph.source_column, "weight".to_string());
        assert_eq!(graph.static_db_data, Vec::new());
        assert_eq!(graph.gtype, GraphType::Simple);
        assert_eq!(graph.timescale, 7*24*3600);
    }
    #[test]
    fn graph_query_parser_2(){
        let query = "/graph/exercise_data/repetitions/exercise_name/Bicep_Curl/multi/30/wset/1/2/3";
        let graph = GraphData::new(query).unwrap();
        assert_eq!(graph.source_table, "exercise_data".to_string());
        assert_eq!(graph.source_column, "repetitions".to_string());
        assert_eq!(graph.static_db_data, Vec::from([("exercise_name".to_string(), "Bicep_Curl".to_string())]));
        assert_eq!(graph.gtype, GraphType::Multi(("wset".to_string(), Vec::from([1_u16.to_string(), 2_u16.to_string(), 3_u16.to_string()]))));
        assert_eq!(graph.timescale, 30*24*3600);
    }
}
