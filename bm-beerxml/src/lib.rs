use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Recipes {
    #[serde(rename = "RECIPE", default)]
    pub recipes: Vec<Recipe>,
}

#[derive(Serialize, Deserialize)]
struct Recipe {
    #[serde(rename = "NAME", default)]
    pub name: String,

    #[serde(rename = "TYPE", default)]
    pub r#type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let file = std::fs::read_to_string(
            "/home/stears/data/me/beer/recipes/Brewfather_BeerXML_DeadPonyClubScaledto20litres_20201213.xml",
        )
        .unwrap();

        let parsed = serde_xml_rs::from_str::<Recipes>(file.as_ref()).unwrap();

        let recipe = &parsed.recipes[0];

        assert_eq!("Dead Pony Club (Scaled to 20 litres)", recipe.name);
    }
}
