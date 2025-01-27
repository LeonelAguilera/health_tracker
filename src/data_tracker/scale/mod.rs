use std::{str::FromStr, time::{SystemTime, UNIX_EPOCH}};

use rusqlite::Connection;

#[derive(Debug)]
pub struct ScaleParameters{
    pub weight: f64,
    pub imc: f64,
    pub body_fat: f64,
    pub visceral_fat: f64,
    pub muscle: f64,
    pub water: f64,
    pub protein: f64,
    pub metabolism: f64,
    pub bone_mass: f64,
    pub hip_diameter: f64,
}

impl ScaleParameters {
    pub fn save_to_db(&self, db: &Connection){
        let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        if let Err(err) = db.execute("INSERT INTO scale_data VALUES (:tim, :weight, :imc, :body_fat, :visceral_fat, :muscle, :water, :protein, :metabolism, :bone_mass, :hip_diameter);", &[
                           (":tim", current_timestamp.to_string().as_str()),          
                           (":weight", self.weight.to_string().as_str()),
                           (":imc", self.imc.to_string().as_str()),
                           (":body_fat", self.body_fat.to_string().as_str()),
                           (":visceral_fat", self.visceral_fat.to_string().as_str()),
                           (":muscle", self.muscle.to_string().as_str()),
                           (":water", self.water.to_string().as_str()),
                           (":protein", self.protein.to_string().as_str()),
                           (":metabolism", self.metabolism.to_string().as_str()),
                           (":bone_mass", self.bone_mass.to_string().as_str()),
                           (":hip_diameter", self.hip_diameter.to_string().as_str()),
        ]){
            println!("Database update failed: {err}");
        }
    }
}

impl FromStr for ScaleParameters {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut weight: Option<f64> = None;
        let mut imc: Option<f64> = None;
        let mut body_fat: Option<f64> = None;
        let mut visceral_fat: Option<f64> = None;
        let mut muscle: Option<f64> = None;
        let mut water: Option<f64> = None;
        let mut protein: Option<f64> = None;
        let mut metabolism: Option<f64> = None;
        let mut bone_mass: Option<f64> = None;
        let mut hip_diameter: Option<f64> = None;

        for parameter in s.split("&"){
            let split_parameter = parameter.split("=").collect::<Vec<_>>();

            match split_parameter[0] {
                "weight" => weight = split_parameter[1].to_string().parse().ok(),
                "imc" => imc = split_parameter[1].to_string().parse().ok(),
                "body_fat" => body_fat = split_parameter[1].to_string().parse().ok(),
                "visceral_fat" => visceral_fat = split_parameter[1].to_string().parse().ok(),
                "muscle" => muscle = split_parameter[1].to_string().parse().ok(),
                "water" => water = split_parameter[1].to_string().parse().ok(),
                "protein" => protein = split_parameter[1].to_string().parse().ok(),
                "metabolism" => metabolism = split_parameter[1].to_string().parse().ok(),
                "bone_mass" => bone_mass = split_parameter[1].to_string().parse().ok(),
                "hip_diameter" => hip_diameter = split_parameter[1].to_string().parse().ok(),
                _ => {},
            }
        }

        Ok(Self{
            weight: match weight{Some(val) => val, None => return Err(())},
            imc: match imc{Some(val) => val, None => return Err(())},
            body_fat: match body_fat{Some(val) => val, None => return Err(())},
            visceral_fat: match visceral_fat{Some(val) => val, None => return Err(())},
            muscle: match muscle{Some(val) => val, None => return Err(())},
            water: match water{Some(val) => val, None => return Err(())},
            protein: match protein{Some(val) => val, None => return Err(())},
            metabolism: match metabolism{Some(val) => val, None => return Err(())},
            bone_mass: match bone_mass{Some(val) => val, None => return Err(())},
            hip_diameter: match hip_diameter{Some(val) => val, None => return Err(())},
        })
    }
}
