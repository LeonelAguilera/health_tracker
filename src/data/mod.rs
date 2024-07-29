use serde_json;

#[derive(Debug)]
pub struct Datos{
    peso: f64,
    grasa_visceral: f64,
    grasa_corporal: f64,
    musculo: f64,
    agua: f64,
    proteina: f64,
    metabolismo_basal: f64,
    masa_osea: f64,
    diametro_cintura: f64,
}

impl Datos {
    pub fn build_from_json(json: serde_json::Value) -> Result<Self, ()>{
        let peso = match json["peso"].as_f64() {
            Some(peso) => Ok(peso),
            None => Err(()),
        }?; 
        let grasa_visceral = match json["grasa_visceral"].as_f64() {
            Some(grasa_visceral) => Ok(grasa_visceral),
            None => Err(()),
        }?;
        let grasa_corporal = match json["grasa_corporal"].as_f64() {
            Some(grasa_corporal) => Ok(grasa_corporal),
            None => Err(()),
        }?;
        let musculo = match json["musculo"].as_f64() {
            Some(musculo) => Ok(musculo),
            None => Err(()),
        }?;
        let agua = match json["agua"].as_f64() {
            Some(agua) => Ok(agua),
            None => Err(()),
        }?;
        let proteina = match json["proteina"].as_f64() {
            Some(proteina) => Ok(proteina),
            None => Err(()),
        }?;
        let metabolismo_basal = match json["metabolismo_basal"].as_f64() {
            Some(metabolismo_basal) => Ok(metabolismo_basal),
            None => Err(()),
        }?;
        let masa_osea = match json["masa_osea"].as_f64() {
            Some(masa_osea) => Ok(masa_osea),
            None => Err(()),
        }?;
        let diametro_cintura = match json["diametro_cintura"].as_f64() {
            Some(diametro_cintura) => Ok(diametro_cintura),
            None => Err(()),
        }?;

        return Ok(Self
        {
            peso,
            grasa_visceral,
            grasa_corporal,
            musculo,
            agua,
            proteina,
            metabolismo_basal,
            masa_osea,
            diametro_cintura,
        });
    }
}
