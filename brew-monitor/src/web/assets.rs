use warp::{reject::Rejection, reply::Reply, Filter};

#[derive(rust_embed::RustEmbed)]
#[folder = "www"]
struct WebContent;

pub fn route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp_embed::embed(&WebContent)
}
