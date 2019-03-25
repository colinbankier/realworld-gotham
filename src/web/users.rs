use failure::Error;
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::helpers::http::response::{create_empty_response, create_response};
use gotham::state::{FromState, State};
use hyper::{Body, StatusCode};
extern crate mime;
use futures::{future, Future, Stream};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::convert::From;
use std::str::from_utf8;

use crate::conduit::users;
use crate::db::Repo;
use crate::models::{NewUser, User};

#[derive(Deserialize, Debug)]
pub struct Registration {
    user: NewUser,
}

#[derive(Serialize)]
pub struct UserResponse {
    user: User,
}

#[derive(Debug)]
pub enum ExtractError {
    SerdeError(serde_json::Error),
    HyperError(hyper::Error),
    Utf8Error(std::str::Utf8Error),
}

fn extract_json<T>(mut state: State) -> impl Future<Item = T, Error = ExtractError>
where
    T: serde::de::DeserializeOwned,
{
    Body::take_from(&mut state)
        .concat2()
        .map_err(ExtractError::HyperError)
        .and_then(|body| {
            let b = body.to_vec();
            from_utf8(&b)
                .map_err(ExtractError::Utf8Error)
                .and_then(|s| serde_json::from_str::<T>(s).map_err(ExtractError::SerdeError))
        })
}

pub fn register(mut state: State) -> Box<HandlerFuture> {
    let repo = Repo::borrow_from(&state).clone();
    // let f = Body::take_from(&mut state)
    //     .concat2()
    //     .map_err(|e| e.into_handler_error())
    //     .and_then(|body| {
    //         let reg = serde_json::from_str::<Registration>(from_utf8(&body.to_vec()).unwrap());
    //         match reg {
    //             Ok(registration) => future::ok(registration),
    //             Err(e) => future::err(e.into_handler_error()),
    //         }
    //     })
    let f = extract_json::<Registration>(state)
        // .map_err(|e| e.into_handler_error(e))
        .and_then(|registration| {
            users::insert(repo, registration.user).map_err(|e| e.into_handler_error())
        })
        .then(|result| match result {
            Ok(user_result) => match user_result {
                Ok(user) => {
                    let body = serde_json::to_string(&user).expect("Failed to serialize user.");
                    let res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
                    future::ok((state, res))
                }
                Err(e) => future::err((state, e.into_handler_error())),
            },
            Err(e) => future::err((state, e.into_handler_error())),
        });
    Box::new(f)
}
