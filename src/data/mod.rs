use serde::{Serialize, Deserialize};
use chrono::{self, DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Datos{
    pub timestamp: Option<i64>,
    peso: Option<f64>,
    grasa_visceral: Option<f64>,
    grasa_corporal: Option<f64>,
    musculo: Option<f64>,
    agua: Option<f64>,
    proteina: Option<f64>,
    metabolismo_basal: Option<f64>,
    masa_osea: Option<f64>,
    diametro_cintura: Option<f64>,
}

impl Datos {
    pub fn is_recent(&self) -> bool
    {
        let current_time = chrono::offset::Utc::now();
        let data_time: DateTime<Utc> = DateTime::from_timestamp(self.timestamp.unwrap(), 0).unwrap();

        return (current_time - data_time).num_days() <= 7;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatosConFecha
{
    pub timestamp: i64,
    pub datos: f64,
}
