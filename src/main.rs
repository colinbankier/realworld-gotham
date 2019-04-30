#[macro_use]
extern crate diesel;

mod auth;
mod conduit;
mod models;
mod schema;
mod web;

#[cfg(test)]
mod test_helpers;

use std::env;

use dotenv::dotenv;
use diesel::pg::PgConnection;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::set::{finalize_pipeline_set, new_pipeline_set};
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;
use gotham_middleware_jwt::JWTMiddleware;
use gotham_middleware_diesel::{self, DieselMiddleware};

const HELLO_ROUTER: &str = "Hello Router!";

pub type Repo = gotham_middleware_diesel::Repo<PgConnection>;

pub fn say_hello(state: State) -> (State, &'static str) {
    (state, HELLO_ROUTER)
}

pub fn router(repo: Repo) -> Router {
    let pipelines = new_pipeline_set();
    let (pipelines, default) =
        pipelines.add(new_pipeline().add(DieselMiddleware::new(repo)).build());
    let (pipelines, authenticated) = pipelines.add(
        new_pipeline()
            // Need to customize realm, as per Guardian.VerifyHeader
            .add(JWTMiddleware::<auth::Claims>::new("secret".as_ref()))
            .build(),
    );
    let pipeline_set = finalize_pipeline_set(pipelines);
    let default_chain = (default, ());
    let auth_chain = (authenticated, default_chain);

    build_router(default_chain, pipeline_set, |route| {
        route.get("/").to(say_hello);
        route.scope("/api", |route| {
            route.post("/users").to(web::users::register);
            route.post("/users/login").to(web::users::login);
            route.with_pipeline_chain(auth_chain, |route| {
                route.get("/user").to(web::users::get_user);
            });
        })
    })
}

pub fn repo() -> Repo {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
Repo::new(&database_url)
}

pub fn main() {
    dotenv().ok();
    env_logger::init();
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);

    gotham::start(addr, router(repo()))
}
