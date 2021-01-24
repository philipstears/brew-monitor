pub mod assets;
pub mod dht22;
pub mod gf;
pub mod recipes;
pub mod tilt;

mod warp_helpers {
    use std::convert::Infallible;
    use warp::Filter;

    pub fn with<T: Clone + Send>(value: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
        warp::any().map(move || value.clone())
    }
}
