use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

type Millilitre = u32;
type Gramme = u32;
type Minute = u32;
type Celsius = u32;

#[derive(Serialize, Deserialize)]
pub struct Recipe {
    #[serde(rename = "batchSize")]
    pub batch_size: Millilitre,
    #[serde(rename = "boilSize")]
    pub boil_size: Millilitre,
    #[serde(rename = "mashSteps")]
    pub mash_steps: Vec<MashStep>,
    #[serde(rename = "boilAdditions")]
    pub boil_additions: Vec<BoilAddition>,
}

#[derive(Serialize, Deserialize)]
pub struct MashStep {
    pub name: String,
    pub time: Minute,
    pub temp: Celsius,
}

#[derive(Serialize, Deserialize)]
pub struct BoilAddition {
    /// The type of the addition.
    pub kind: BoilAdditionType,

    /// The name of the addition to add, e.g. "Mosaic".
    pub name: String,

    /// The mass of the boil addition.
    pub mass: Gramme,

    /// Time from the end of the boil.
    pub time: Minute,
}

#[derive(Serialize, Deserialize)]
pub enum BoilAdditionType {
    Hop,
    YeastNutrient,
    Other {
        description: String,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
