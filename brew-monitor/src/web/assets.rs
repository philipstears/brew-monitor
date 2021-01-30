use warp::{filters::BoxedFilter, reply::Reply, Filter};

#[derive(rust_embed::RustEmbed)]
#[folder = "www"]
struct WebContent;

pub fn route() -> BoxedFilter<(impl Reply,)> {
    warp_embed::embed(&WebContent).boxed()
}
