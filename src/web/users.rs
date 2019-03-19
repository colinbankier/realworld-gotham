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

pub fn register(mut state: State) -> Box<HandlerFuture> {
    let repo = Repo::try_borrow_from(&state).unwrap_or_else(|| panic!("No repo found in state."));
    // if let None = repo {
    //     return Box::new(future::ok((state, create_empty_response(&state, StatusCode::INTERNAL_SERVER_ERROR))));
    // }
    let empty = create_empty_response(&state, StatusCode::OK);
    let f = Body::take_from(&mut state)
        .concat2()
        .then(|body| match body {
            Ok(valid_body) => {
                let reg =
                    serde_json::from_str::<Registration>(from_utf8(&valid_body.to_vec()).unwrap());
                future::ok(reg)
            }
            Err(e) => future::err(e),
        })
        .then(|body| match body {
            Ok(valid_body) => {
                let res = create_empty_response(&state, StatusCode::OK);
                future::ok((state, res))
            }
            Err(e) => future::err((state, e.into_handler_error())),
        });
    Box::new(f)

    // let res = f.and_then(move |registration|{
    //     users::insert(repo.clone(), registration.user)
    //     .map_err(|e| e.into_handler_error())}
    //         // .and_then(|user_result| match user_result {
    //         //     Ok(user) => {
    //         // let json = serde_json::to_string(&UserResponse { user }).expect("Error encoding json");
    //         //     let res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, json);
    //         //     Box::new(future::ok((state, res)))
    //         //     },
    //         //     Err(e) => Box::new(future::err((state, e.into_handler_error())))
    //         // }
    //         // )
    //         // .map_err(|e| ( state, e.into_handler_error() ))
    //     );
    // Box::new(res.then(|result| match result {
    //     Ok(success) => Ok((state, empty)),
    //     Err(e) => Err((state, e.into_handler_error()))
    // }))
    // res
}
// pub fn register(
//     repo: AppData<Repo>,
//     registration: Json<Registration>,
// ) -> Result<Json<UserResponse>, StatusCode> {
//     let result = await! { users::insert(repo.clone(), registration.0.user) };

//     result
//         .map(|user| Json(UserResponse { user }))
//         .map_err(|e| diesel_error(&e))
// }
