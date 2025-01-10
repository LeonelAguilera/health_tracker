use rusqlite::{Connection, Error};

const ALLOWED_COLUMN_NAMES: &[&str] = &["weight", "imc", "body_fat", "visceral_fat", "muscle", "water", "protein", "metabolism", "bone_mass", "hip_diameter"];
const ALLOWED_TABLES_NAMES: &[&str] = &["scale_data"];

pub fn graph_maker(db: &Connection, key: &str, table_name: &str) -> Result<Vec<(String, f64)>, String>{
    if !ALLOWED_COLUMN_NAMES.contains(&key){
        return Err("Columna no válida".to_string());
    }
    if !ALLOWED_TABLES_NAMES.contains(&table_name){
        return Err("Tabla no válida".to_string());
    }

    let query = format!("SELECT timev, {}
                        FROM {}
                        WHERE timev >= DATETIME('now', '-7 days')
                        UNION
                        SELECT timev, {}
                        FROM(
                        SELECT timev, {}
                        FROM {}
                        WHERE timev < DATETIME('now', '-7 days')
                        ORDER BY timev DESC
                        LIMIT 1);", key, table_name, key, key, table_name);
    let mut statement = db.prepare(&query).unwrap();

    let data = statement.query_map([], |row|{
        let date = row.get::<usize, String>(0);
        let value = row.get::<usize, f64>(1);

        match (date, value) {
            (Ok(date), Ok(value)) => Ok((date, value)),
            _ => Err(Error::InvalidQuery),
        }
    });

    let mut graph = Vec::new();
    let data = data.unwrap();

    for dato in data{
        graph.push(dato.unwrap());
    }

    return Ok(graph);
}
