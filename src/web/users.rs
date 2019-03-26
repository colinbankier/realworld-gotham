use failure::Error;
use gotham::handler::{HandlerError, HandlerFuture, IntoHandlerError};
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

fn bad_request<E>(e: E) -> HandlerError
where
    E: std::error::Error + Send + 'static,
{
    e.into_handler_error().with_status(StatusCode::BAD_REQUEST)
}

fn extract_json<T>(state: &mut State) -> impl Future<Item = T, Error = HandlerError>
where
    T: serde::de::DeserializeOwned,
{
    Body::take_from(state)
        .concat2()
        .map_err(bad_request)
        .and_then(|body| {
            let b = body.to_vec();
            from_utf8(&b)
                .map_err(bad_request)
                .and_then(|s| serde_json::from_str::<T>(s).map_err(bad_request))
        })
}

pub fn register(mut state: State) -> Box<HandlerFuture> {
    let repo = Repo::borrow_from(&state).clone();
    let f = extract_json::<Registration>(&mut state)
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

#[cfg(test)]
mod tests {
    use crate::models::NewUser;
    use crate::test_helpers::generate;
    use gotham::test::TestServer;
    use crate::db::Repo;
    use serde_json::{json, Value};
    use hyper::StatusCode;
    use crate::router;

    #[test]
   fn register_and_login() {

        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let server = TestServer::new(Repo::new());
        let user = generate::new_user();

        await! {register_user(&server, &user)};
        let token = await! {login_user(&server, &user)};
        let user_details = await! { get_user_details(&server, &token)};

        assert_eq!(user_details["user"]["username"], user.username);
        assert_eq!(user_details["user"]["email"], user.email);
    }
}