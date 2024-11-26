use rusqlite::Connection;

pub fn get_scale_database() -> Connection{
    let db = Connection::open("databases/scale_data.db").expect("Could not open the scale data database");
    let _ = db.execute("CREATE TABLE IF NOT EXISTS scale_data (
            time TIMESTAMP primary key,
            peso FLOAT,
            grasa_visceral FLOAT,
            grasa_corporal FLOAT,
            musculo FLOAT,
            agua FLOAT,
            proteina FLOAT,
            metabolismo_basal FLOAT,
            masa_osea FLOAT,
            diametro_cintura FLOAT
            )",
            []).expect("Could not create table for scale data");
    return db;
}
