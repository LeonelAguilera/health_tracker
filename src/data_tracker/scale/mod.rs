use std::str::FromStr;

#[derive(Debug)]
pub struct ScaleParameters{
    weight: f64,
    imc: f64,
    body_fat: f64,
    visceral_fat: f64,
    muscle: f64,
    water: f64,
    protein: f64,
    metabolism: f64,
    bone_mass: f64,
    hip_diameter: f64,
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
