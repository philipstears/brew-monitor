use bm_beerxml;
use bm_db::DB;
use bm_recipe;
use chrono::{DateTime, Utc};
use warp::{
    reject::Rejection,
    reply::{Reply, Response},
    Filter,
};

struct NewRecipeRequest {
    name: String,
}

struct NewRecipeVersionRequest {
    data: bm_recipe::Recipe,
}

struct ExistingRecipe {
    name: String,
    created_on: DateTime<Utc>,
}

struct ExistingRecipeVersion {
    created_on: DateTime<Utc>,
    data: bm_recipe::Recipe,
}

pub fn route(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path::path("recipes")
        .and(resources::recipes(db.clone()).or(resources::recipe(db.clone())).recover(resources::handle_rejection))
}

mod resources {
    use super::*;

    pub(super) fn recipes(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let get = warp::path::end()
            .and(warp::filters::method::get())
            .and(with_db(db.clone()))
            .and_then(handlers::recipes_get);

        let post = warp::path::end()
            .and(warp::filters::method::post())
            .and(warp::body::content_length_limit(65_536))
            // TODO: this returns 400 if it doesn't match, rather than 406
            .and(require_xml())
            .and(warp::body::bytes())
            .and(with_db(db.clone()))
            .and_then(handlers::recipes_import);

        get.or(post)
    }

    pub(super) fn recipe(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let get = warp::path!(String)
            .and(warp::filters::method::get())
            .and(with_db(db.clone()))
            .and_then(handlers::recipe_get);

        let put = warp::path!(String)
            .and(warp::filters::method::put())
            .and(warp::body::content_length_limit(65_536))
            //.and(warp::body::json())
            .and(with_db(db.clone()))
            .and_then(handlers::recipe_upsert);

        get.or(put)
    }

    fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    fn require_xml() -> impl Filter<Extract = (), Error = Rejection> + Clone {
        warp::header("content-type")
            .and_then(|content_type: String| async move {
                if content_type == "text/xml" {
                    Ok(())
                } else {
                    Err(warp::reject::custom(NotAcceptableRejection))
                }
            })
            .untuple_one()
    }

    #[derive(Debug)]
    struct NotAcceptableRejection;

    impl warp::reject::Reject for NotAcceptableRejection {}

    pub(super) async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
        use warp::http::StatusCode;

        let code = if err.is_not_found() {
            StatusCode::NOT_FOUND
        } else if let Some(NotAcceptableRejection) = err.find() {
            StatusCode::NOT_ACCEPTABLE
        } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
            // // This error happens if the body could not be deserialized correctly
            // // We can use the cause to analyze the error and customize the error message
            // message = match e.source() {
            //     Some(cause) => {
            //         if cause.to_string().contains("denom") {
            //             "FIELD_ERROR: denom"
            //         } else {
            //             "BAD_REQUEST"
            //         }
            //     }
            //     None => "BAD_REQUEST",
            // };
            StatusCode::BAD_REQUEST
        } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
            // We can handle a specific error, here METHOD_NOT_ALLOWED,
            // and render it however we want
            StatusCode::METHOD_NOT_ALLOWED
        } else {
            // We should have expected this... Just log and say its a 500
            eprintln!("unhandled rejection: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        };

        Ok(warp::reply::with_status(warp::reply::reply(), code))
    }
}

mod handlers {
    use super::*;

    pub(super) async fn recipe_get(name: String, db: DB) -> Result<Response, Rejection> {
        let reply = match db.recipe().get_recipe_latest(&name) {
            Ok(Some(info)) => warp::reply::json(&info.version_data).into_response(),
            Ok(None) => {
                eprintln!("Couldn't find recipe {}", name);
                warp::reply::with_status(warp::reply::reply(), warp::http::StatusCode::NOT_FOUND).into_response()
            }
            Err(err) => {
                eprintln!("Couldn't get recipe {}: {:?}", name, err);
                warp::reply::with_status(warp::reply::reply(), warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        };

        Ok(reply)
    }

    pub(super) async fn recipe_upsert(_alias: String, _db: DB) -> Result<Response, Rejection> {
        let reply = warp::reply::with_status(warp::reply::reply(), warp::http::StatusCode::CREATED).into_response();
        Ok(reply)
    }

    pub(super) async fn recipes_get(db: DB) -> Result<Response, Rejection> {
        let reply = match db.recipe().get_recipes() {
            Ok(recipes) => warp::reply::json(&recipes).into_response(),
            Err(err) => {
                eprintln!("Couldn't get recipes: {:?}", err);
                warp::reply::with_status(warp::reply::reply(), warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        };

        Ok(reply)
    }

    pub(super) async fn recipes_import(data: bytes::Bytes, db: DB) -> Result<Response, Rejection> {
        let recipes_in: bm_beerxml::Recipes = serde_xml_rs::from_reader(data.as_ref()).unwrap();

        for recipe_in in recipes_in.recipes {
            let recipe_out = bm_recipe::Recipe {
                batch_size: (recipe_in.batch_size * 1_000.0).trunc() as u32,
                boil_size: (recipe_in.boil_size * 1_000.0).trunc() as u32,
                boil_time: recipe_in.boil_time,
                mash_steps: {
                    let mut mash_steps = Vec::with_capacity(recipe_in.mash.steps.steps.len());

                    for mash_step_in in recipe_in.mash.steps.steps.iter() {
                        let mash_step_out = bm_recipe::MashStep {
                            name: mash_step_in.name.clone(),
                            time: mash_step_in.time.into(),
                            temp: mash_step_in.temp.into(),
                        };

                        mash_steps.push(mash_step_out);
                    }

                    mash_steps
                },
                boil_additions: {
                    let mut boil_additions = Vec::with_capacity(recipe_in.hops.hops.len());

                    for hop_in in recipe_in.hops.hops.iter().filter(|hop| hop.r#use == bm_beerxml::HopUse::Boil) {
                        let mash_step_out = bm_recipe::BoilAddition {
                            name: hop_in.name.clone(),
                            amount: bm_recipe::Amount::Mass((hop_in.amount * 1_000.0).trunc() as u32),
                            time: hop_in.time.into(),
                            kind: bm_recipe::BoilAdditionType::Hop,
                        };

                        boil_additions.push(mash_step_out);
                    }

                    for misc_in in recipe_in.miscs.miscs.iter().filter(|misc| misc.r#use == bm_beerxml::MiscUse::Boil) {
                        let mash_step_out = bm_recipe::BoilAddition {
                            name: misc_in.name.clone(),
                            amount: if misc_in.amount_is_weight {
                                bm_recipe::Amount::Mass((misc_in.amount * 1_000.0).trunc() as u32)
                            } else {
                                bm_recipe::Amount::Volume((misc_in.amount * 1_000.0).trunc() as u32)
                            },
                            time: misc_in.time.into(),
                            kind: if misc_in.name.to_lowercase() == "yeast nutrient" {
                                bm_recipe::BoilAdditionType::YeastNutrient
                            } else {
                                bm_recipe::BoilAdditionType::Other {
                                    description: misc_in.name.clone(),
                                }
                            },
                        };

                        boil_additions.push(mash_step_out);
                    }

                    boil_additions.sort_by(|a, b| b.time.cmp(&a.time));

                    boil_additions
                },
                fermentables: {
                    let mut result = Vec::with_capacity(recipe_in.fermentables.fermentables.len());

                    for fermentable_in in recipe_in.fermentables.fermentables.iter() {
                        let fermentable_out = bm_recipe::Fermentable {
                            name: fermentable_in.name.clone(),
                            mass: (fermentable_in.amount * 1000.0).trunc() as u32,
                        };

                        result.push(fermentable_out);
                    }

                    result
                },
            };

            let grain_bill = recipe_in.fermentables.fermentables.iter().map(|f| f.amount).sum();
            let mash_water = bm_grainfather::calc::mash_water_metric(grain_bill);
            let sparge_water = bm_grainfather::calc::sparge_water_metric(recipe_in.batch_size, grain_bill);

            println!("Mash: {}l, Sparge: {}l", mash_water, sparge_water);

            println!("Got {:#?}", recipe_out);

            db.recipe().ensure(recipe_in.name.as_ref()).unwrap();

            db.recipe().insert_version(recipe_in.name.as_ref(), &recipe_out).unwrap();
        }

        let reply = warp::reply::with_status(warp::reply::reply(), warp::http::StatusCode::CREATED).into_response();
        Ok(reply)
    }
}
