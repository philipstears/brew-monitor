use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RECIPES")]
pub struct Recipes {
    #[serde(rename = "RECIPE")]
    pub recipes: Vec<Recipe>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RECIPE")]
pub struct Recipe {
    #[serde(rename = "NAME")]
    pub name: String,

    #[serde(rename = "BREWER")]
    pub brewer: String,

    #[serde(rename = "BATCH_SIZE")]
    pub batch_size: f64,

    #[serde(rename = "BOIL_SIZE")]
    pub boil_size: f64,

    #[serde(rename = "TYPE")]
    pub r#type: String,

    #[serde(rename = "OG", default)]
    pub original_gravity: Option<f64>,

    #[serde(rename = "FG", default)]
    pub final_gravity: Option<f64>,

    // NOTE: this is defined as a percentage in the spec, but brewfather emits
    // a string suffixed with a percentage symbol
    #[serde(rename = "ABV", default)]
    pub abv: Option<String>,

    #[serde(rename = "IBU", default)]
    pub ibu: Option<f64>,

    #[serde(rename = "HOPS")]
    pub hops: Hops,

    #[serde(rename = "MISCS")]
    pub miscs: Miscs,

    #[serde(rename = "MASH")]
    pub mash: Mash,

    #[serde(rename = "EST_OG", default)]
    pub estimated_original_gravity: Option<String>,

    #[serde(rename = "EST_FG", default)]
    pub estimated_final_gravity: Option<String>,

    // NOTE: this is defined as a percentage in the spec, but brewfather emits
    // a string suffixed with a percentage symbol
    #[serde(rename = "EST_ABV", default)]
    pub estimated_abv: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hops {
    #[serde(rename = "HOP")]
    pub hops: Vec<Hop>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum HopUse {
    #[serde(rename = "Boil")]
    Boil,

    #[serde(rename = "Dry Hop")]
    DryHop,

    #[serde(rename = "Mash")]
    Mash,

    #[serde(rename = "Aroma")]
    Aroma,

    #[serde(rename = "First Wort")]
    FirstWort,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hop {
    #[serde(rename = "NAME")]
    pub name: String,

    #[serde(rename = "USE")]
    pub r#use: HopUse,

    #[serde(rename = "TIME")]
    pub time: u16,

    #[serde(rename = "AMOUNT")]
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Miscs {
    #[serde(rename = "MISC")]
    pub miscs: Vec<Misc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Misc {
    #[serde(rename = "NAME")]
    pub name: String,

    #[serde(rename = "USE")]
    pub r#use: MiscUse,

    #[serde(rename = "TIME")]
    pub time: u16,

    #[serde(rename = "AMOUNT")]
    pub amount: f64,

    #[serde(rename = "AMOUNT_IS_WEIGHT")]
    pub amount_is_weight: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum MiscUse {
    #[serde(rename = "Boil")]
    Boil,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mash {
    #[serde(rename = "MASH_STEPS")]
    pub steps: MashSteps,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MashSteps {
    #[serde(rename = "MASH_STEP")]
    pub steps: Vec<MashStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MashStep {
    #[serde(rename = "NAME")]
    pub name: String,

    #[serde(rename = "STEP_TIME")]
    pub time: u16,

    #[serde(rename = "STEP_TEMP")]
    pub temp: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    const GF_XML: &[u8] = include_bytes!("../test-data/dpc-grainfather.xml");
    const BF_XML: &[u8] = include_bytes!("../test-data/dpc-brewfather.xml");

    #[test]
    fn brewfather_example() {
        let parsed: Recipes = serde_xml_rs::from_reader(BF_XML).unwrap();
        let recipe = &parsed.recipes[0];
        assert_eq!("Dead Pony Club (Scaled to 20 litres)", recipe.name);
        assert_eq!(1, recipe.mash.steps.steps.len());
        assert_eq!(7, recipe.hops.hops.len());
    }

    #[test]
    fn grainfather_example() {
        let parsed: Recipes = serde_xml_rs::from_reader(GF_XML).unwrap();
        let recipe = &parsed.recipes[0];
        assert_eq!("Dead Pony Club (Brewdog)", recipe.name);
        assert_eq!(2, recipe.mash.steps.steps.len());
        assert_eq!(7, recipe.hops.hops.len());
    }
}
