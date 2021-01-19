use bm_recipe;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result, Row, NO_PARAMS};
use serde::{Deserialize, Serialize};

use super::WrappedConnection;

pub type RecipeId = i64;
pub type Version = u32;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeInfo {
    pub id: RecipeId,
    pub name: String,
    pub created_on: DateTime<Utc>,
    pub latest_version: Option<Version>,
}

#[derive(Clone)]
pub struct RecipeData(WrappedConnection);

impl RecipeData {
    pub(super) fn new(connection: WrappedConnection) -> Self {
        Self(connection)
    }

    pub fn ensure(&self, name: &str) -> Result<()> {
        let connection = self.0.lock_or_panic();

        connection.execute(
            "
            insert into recipes (name,created_on)
            values (?,?)
            on conflict(name) do nothing;
            ",
            params![name, Utc::now().timestamp()],
        )?;

        Ok(())
    }

    pub fn get_recipes(&self) -> Result<Vec<RecipeInfo>> {
        let connection = self.0.lock_or_panic();
        recipes(&connection)
    }

    pub fn get_recipe(&self, name: &str) -> Result<Option<bm_recipe::Recipe>> {
        let connection = self.0.lock_or_panic();

        let mut statement = connection.prepare(
            "
            select data from recipe_versions
            inner join recipes
            on recipe_versions.recipe_id = recipes.id
            and recipe_versions.version_id = recipes.latest_version
            where recipes.name = ?
            ",
        )?;

        statement.query_row(params![name], map_recipe_version_data).optional()
    }

    pub fn insert_version(&self, name: &str, recipe: &bm_recipe::Recipe) -> Result<Version> {
        let mut connection = self.0.lock_or_panic();
        let transaction = connection.transaction()?;

        let recipe_info = recipe_details(&transaction, name)?;

        if let Some(previous_version) = recipe_info.latest_version {
            let previous_version_recipe = recipe_version(&transaction, recipe_info.id, previous_version)?;

            if &previous_version_recipe == recipe {
                return Ok(previous_version);
            }
        }

        let new_version_recipe = serde_json::to_string(recipe).unwrap();
        let new_version = recipe_info.latest_version.map(|v| v + 1).unwrap_or(0);

        // Insert the version
        transaction.execute(
            "
            insert into recipe_versions (recipe_id,version_id,created_on,data)
            values (?,?,?,?)
            ",
            params![recipe_info.id, new_version, Utc::now().timestamp(), new_version_recipe],
        )?;

        // Update the latest
        transaction.execute(
            "
            update recipes set latest_version=?
            where id=?
            ",
            params![new_version, recipe_info.id],
        )?;

        transaction.commit()?;

        Ok(new_version)
    }
}

fn recipe_version(connection: &Connection, recipe_id: RecipeId, version: Version) -> Result<bm_recipe::Recipe> {
    let mut statement =
        connection.prepare("select data from recipe_versions where recipe_id = ? AND version_id = ?")?;

    statement.query_row(params![recipe_id, version], map_recipe_version_data)
}

fn recipe_details(connection: &Connection, name: &str) -> Result<RecipeInfo> {
    let mut statement = connection.prepare("select id,name,created_on,latest_version from recipes where name = ?")?;
    statement.query_row(params![name], map_recipe_row)
}

fn recipes(connection: &Connection) -> Result<Vec<RecipeInfo>> {
    let mut statement = connection.prepare("select id,name,created_on,latest_version from recipes")?;
    let mut results = Vec::new();

    for result in statement.query_map(NO_PARAMS, map_recipe_row)? {
        results.push(result?)
    }

    Ok(results)
}

fn map_recipe_row(row: &Row) -> Result<RecipeInfo> {
    Ok(RecipeInfo {
        id: row.get(0)?,
        name: row.get(1)?,
        created_on: Utc.timestamp(row.get(2)?, 0),
        latest_version: row.get(3)?,
    })
}

fn map_recipe_version_data(row: &Row) -> Result<bm_recipe::Recipe> {
    let data: String = row.get(0)?;
    Ok(serde_json::from_str::<bm_recipe::Recipe>(&data).unwrap())
}
