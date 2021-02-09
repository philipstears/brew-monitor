use bm_recipe;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result, Row, NO_PARAMS};
use serde::{Deserialize, Serialize};

use super::WrappedConnection;

pub type RecipeId = i64;
pub type RecipeVersion = u32;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeInfo {
    pub id: RecipeId,
    pub name: String,
    pub created_on: DateTime<Utc>,
    pub latest_version: Option<RecipeVersion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeVersionInfo {
    pub id: RecipeId,
    pub name: String,
    pub created_on: DateTime<Utc>,
    pub version: RecipeVersion,
    pub version_data: bm_recipe::Recipe,
    pub version_created_on: DateTime<Utc>,
}

impl RecipeVersionInfo {
    fn from_row(row: &Row) -> Result<RecipeVersionInfo> {
        let version_data: String = row.get(4)?;

        Ok(RecipeVersionInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            created_on: Utc.timestamp(row.get(2)?, 0),
            version: row.get(3)?,
            version_data: serde_json::from_str::<bm_recipe::Recipe>(&version_data).unwrap(),
            version_created_on: Utc.timestamp(row.get(5)?, 0),
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RecipeSelector<'a> {
    ByName(&'a str),
    ById(RecipeId),
}

#[derive(Debug, Copy, Clone)]
pub enum RecipeVersionSelector {
    Latest,
    SpecificVersion(RecipeVersion),
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

    pub fn get_recipe(
        &self,
        recipe: RecipeSelector,
        version: RecipeVersionSelector,
    ) -> Result<Option<RecipeVersionInfo>> {
        let connection = self.0.lock_or_panic();

        match (recipe, version) {
            (RecipeSelector::ByName(name), RecipeVersionSelector::Latest) => {
                let mut statement = connection.prepare(
                    "
                    select
                      r.id,r.name,r.created_on,
                      v.version_id,v.data,v.created_on
                    from recipe_versions v
                    inner join recipes r
                    on v.recipe_id = r.id
                    and v.version_id = r.latest_version
                    where r.name = ?
                    ",
                )?;

                statement.query_row(params![name], RecipeVersionInfo::from_row).optional()
            }

            (RecipeSelector::ById(id), RecipeVersionSelector::Latest) => {
                let mut statement = connection.prepare(
                    "
                    select
                      r.id,r.name,r.created_on,
                      v.version_id,v.data,v.created_on
                    from recipe_versions v
                    inner join recipes r
                    on v.recipe_id = r.id
                    and v.version_id = r.latest_version
                    where r.id = ?
                    ",
                )?;

                statement.query_row(params![id], RecipeVersionInfo::from_row).optional()
            }

            (RecipeSelector::ByName(name), RecipeVersionSelector::SpecificVersion(version)) => {
                let mut statement = connection.prepare(
                    "
                    select
                      r.id,r.name,r.created_on,
                      v.version_id,v.data,v.created_on
                    from recipe_versions v
                    inner join recipes r
                    on v.recipe_id = r.id
                    where r.name = ?
                    and v.version_id = ?
                    ",
                )?;

                statement.query_row(params![name, version], RecipeVersionInfo::from_row).optional()
            }

            (RecipeSelector::ById(id), RecipeVersionSelector::SpecificVersion(version)) => {
                let mut statement = connection.prepare(
                    "
                    select
                      r.id,r.name,r.created_on,
                      v.version_id,v.data,v.created_on
                    from recipe_versions v
                    inner join recipes r
                    on v.recipe_id = r.id
                    where r.id = ?
                    and v.version_id = ?
                    ",
                )?;

                statement.query_row(params![id, version], RecipeVersionInfo::from_row).optional()
            }
        }
    }

    pub fn insert_version(&self, name: &str, recipe: &bm_recipe::Recipe) -> Result<RecipeVersion> {
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

fn recipe_version(connection: &Connection, recipe_id: RecipeId, version: RecipeVersion) -> Result<bm_recipe::Recipe> {
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
