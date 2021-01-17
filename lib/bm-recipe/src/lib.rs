use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

type Millilitre = u32;
type Gramme = u32;
type Minute = u32;
type Celsius = u32;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    #[serde(rename = "batchSize")]
    pub batch_size: Millilitre,
    #[serde(rename = "boilSize")]
    pub boil_size: Millilitre,
    #[serde(rename = "mashSteps")]
    pub mash_steps: Vec<MashStep>,
    #[serde(rename = "boilAdditions")]
    pub boil_additions: Vec<BoilAddition>,
    pub fermentables: Vec<Fermentable>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MashStep {
    pub name: String,
    pub time: Minute,
    pub temp: Celsius,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BoilAddition {
    /// The type of the addition.
    pub kind: BoilAdditionType,

    /// The name of the addition to add, e.g. "Mosaic".
    pub name: String,

    /// The amount of the boil addition.
    pub amount: Amount,

    /// Time from the end of the boil.
    pub time: Minute,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Amount {
    Mass(Gramme),
    Volume(Millilitre),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum BoilAdditionType {
    Hop,
    YeastNutrient,
    Other {
        description: String,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Fermentable {
    pub name: String,
    pub mass: Gramme,
}
