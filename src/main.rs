#[macro_use]
extern crate diesel;

mod conduit;
mod db;
mod diesel_middleware;
mod models;
mod schema;
mod web;

#[cfg(test)]
mod test_helpers;

use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::*;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;

use db::Repo;
use diesel_middleware::DieselMiddleware;

const HELLO_ROUTER: &str = "Hello Router!";

pub fn say_hello(state: State) -> (State, &'static str) {
    (state, HELLO_ROUTER)
}

pub fn router(repo: Repo) -> Router {
    let (chain, pipelines) =
        single_pipeline(new_pipeline().add(DieselMiddleware::new(repo)).build());

    build_router(chain, pipelines, |route| {
        route.get("/").to(say_hello);
        route.scope("/api", |route| {
            route.post("/users").to(web::users::register);
            route.post("/users/login").to(web::users::login);
        })
    })
}

pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);

    gotham::start(addr, router(Repo::new()))
}
