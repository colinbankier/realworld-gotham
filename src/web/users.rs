use gotham::handler::{HandlerError, HandlerFuture, IntoHandlerError};
use gotham::helpers::http::response::{create_empty_response, create_response};
use gotham::state::{FromState, State};
use gotham_middleware_jwt::AuthorizationToken;
use hyper::{Body, StatusCode};
extern crate mime;
use futures::{future, Future, Stream};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::str::from_utf8;

use crate::auth::{encode_token, Claims};
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

pub fn login(mut state: State) -> Box<HandlerFuture> {
    let repo = Repo::borrow_from(&state).clone();
    let f = extract_json::<NewUser>(&mut state)
        .and_then(move |user| {
            users::find_by_email_password(repo, user.email, user.password)
                .map_err(|e| e.into_handler_error())
        })
        .then(|result| match result {
            Ok(Ok(user)) => {
                let response = UserResponse {
                    user: User {
                        token: Some(encode_token(user.id)),
                        ..user
                    },
                };
                let body = serde_json::to_string(&response).expect("Failed to serialize user.");
                let res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
                future::ok((state, res))
            }
            Ok(Err(diesel::result::Error::NotFound)) => {
                let res = create_empty_response(&state, StatusCode::UNAUTHORIZED);
                future::ok((state, res))
            }
            Ok(Err(e)) => future::err((state, e.into_handler_error())),
            Err(e) => future::err((state, e.into_handler_error())),
        });
    Box::new(f)
}

pub fn get_user(mut state: State) -> Box<HandlerFuture> {
    let repo = Repo::borrow_from(&state).clone();
    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let results = users::find(repo.clone(), token.0.claims.user_id()).then(|result| match result {
        Ok(Ok(user)) => {
            let response = UserResponse { user };
            let body = serde_json::to_string(&response).expect("Failed to serialize user.");
            let res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
            future::ok((state, res))
        }
        Ok(Err(diesel::result::Error::NotFound)) => {
            let res = create_empty_response(&state, StatusCode::UNAUTHORIZED);
            future::ok((state, res))
        }
        Ok(Err(e)) => future::err((state, e.into_handler_error())),
        Err(e) => future::err((state, e.into_handler_error())),
    });
    Box::new(results)
}

#[cfg(test)]
mod tests {
    use crate::db::Repo;
    use crate::models::NewUser;
    use crate::router;
    use crate::test_helpers::generate;
    use futures::future::Future;
    use futures::stream::Stream;
    use gotham::test::{TestResponse, TestServer};
    use hyper::{header::HeaderValue, StatusCode};
    use serde_json::{json, Value};

    use std::str::from_utf8;

    #[test]
    fn register_and_login() {
        let server = TestServer::new(router(Repo::new())).unwrap();
        let user = generate::new_user();

        register_user(&server, &user);
        let token = login_user(&server, &user);
        let user_details = get_user_details(&server, &token);

        assert_eq!(user_details["user"]["username"], user.username);
        assert_eq!(user_details["user"]["email"], user.email);
    }

    pub fn response_json(res: TestResponse) -> Value {
        let body = res.read_body().unwrap();
        serde_json::from_str(from_utf8(&body).unwrap()).expect("Could not parse body.")
    }

    fn register_user<'a>(server: &'a TestServer, user: &'a NewUser) -> Value {
        let res = server
            .client()
            .post(
                "/api/users",
                json!({
                    "user": {
                        "email": user.email,
                        "password": user.password,
                        "username": user.username,
                    }
                })
                .to_string(),
                mime::APPLICATION_JSON,
            )
            .perform()
            .unwrap();
        response_json(res)
    }

    fn login_user<'a>(server: &'a TestServer, user: &'a NewUser) -> String {
        let res = server
            .client()
            .post(
                "/api/users/login",
                json!({
                    "user": {
                        "email": user.email,
                        "password": user.password,
                    }
                })
                .to_string(),
                mime::APPLICATION_JSON,
            )
            .perform()
            .unwrap();
        assert_eq!(res.status(), 200);

        let response_json = response_json(res);

        assert!(response_json["user"]["token"].is_string());
        response_json["user"]["token"]
            .as_str()
            .expect("Token not found")
            .to_string()
    }

    fn get_user_details<'a>(server: &'a TestServer, token: &'a String) -> Value {
        let res = server
            .client()
            .get("/api/user")
            .with_header(
                "Authorization",
                HeaderValue::from_str(&format!("token: {}", token)).unwrap(),
            )
            .perform()
            .unwrap();
        assert_eq!(res.status(), 200);
        response_json(res)
    }

}
