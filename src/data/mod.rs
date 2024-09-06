use serde::{Serialize, Deserialize};
use chrono::{self, DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Datos{
    pub timestamp: Option<i64>,
    pub peso: Option<f64>,
    pub grasa_visceral: Option<f64>,
    pub grasa_corporal: Option<f64>,
    pub musculo: Option<f64>,
    pub agua: Option<f64>,
    pub proteina: Option<f64>,
    pub metabolismo_basal: Option<f64>,
    pub masa_osea: Option<f64>,
    pub diametro_cintura: Option<f64>,
}

impl Datos {
    pub fn is_recent(&self) -> bool
    {
        let current_time = chrono::offset::Utc::now();
        let data_time: DateTime<Utc> = DateTime::from_timestamp(self.timestamp.unwrap(), 0).unwrap();

        return (current_time - data_time).num_days() <= 7;
    }
    pub fn deserialize(data: String) -> Self
    {
        Datos{
            timestamp: None,
            peso: get_data_from_key(&data, "peso"),
            grasa_visceral: get_data_from_key(&data, "grasa_visceral"),
            grasa_corporal: get_data_from_key(&data, "grasa_corporal"),
            musculo: get_data_from_key(&data, "musculo"),
            agua: get_data_from_key(&data, "agua"),
            proteina: get_data_from_key(&data, "proteina"),
            metabolismo_basal: get_data_from_key(&data, "metabolismo"),
            masa_osea: get_data_from_key(&data, "masa_osea"),
            diametro_cintura: get_data_from_key(&data, "diametro_cintura"),
        }
    }
}

fn get_data_from_key(data: &String, key: &str) -> Option<f64>
{
    let key_pos = data.find(key);

    if key_pos == None{
        return None;
    }
    let key_pos = key_pos.unwrap();
    
    let working_data = &data[key_pos..];
    let num_start = working_data.find("=").expect("Malformed packet\nworking data: {working_data}\nkey: {key}\n") + 1;
    let num_end = working_data.find("&").expect("Malformed packet\nworking data: {working_data}\nkey: {key}\n");

    let my_number = &working_data[num_start..num_end];
    return match my_number.parse::<f64>() {
        Ok(value) => Some(value),
        _ => None,
    }
}

pub struct DatosConFecha
{
    pub timestamp: i64,
    pub datos: f64,
}
