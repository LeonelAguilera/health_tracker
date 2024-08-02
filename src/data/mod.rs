use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Datos{
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub peso: Option<f64>,
    grasa_visceral: Option<f64>,
    grasa_corporal: Option<f64>,
    musculo: Option<f64>,
    agua: Option<f64>,
    proteina: Option<f64>,
    metabolismo_basal: Option<f64>,
    masa_osea: Option<f64>,
    diametro_cintura: Option<f64>,
}
