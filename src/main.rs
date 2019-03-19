#[macro_use]
extern crate diesel;

mod conduit;
mod db;
mod models;
mod schema;
mod web;

#[cfg(test)]
mod test_helpers;

use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;

const HELLO_ROUTER: &str = "Hello Router!";

pub fn say_hello(state: State) -> (State, &'static str) {
    (state, HELLO_ROUTER)
}

fn router() -> Router {
    build_simple_router(|route| {
        route.get("/").to(say_hello);
    })
}

pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);

    gotham::start(addr, router())
}
