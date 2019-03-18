use crate::db::Repo;
use crate::models::{NewUser, UpdateUser, User};
use crate::schema::users;

use diesel::prelude::*;
use diesel::result::Error;
use futures::Future;
use tokio_threadpool::BlockingError;

pub fn insert(
    repo: Repo,
    user: NewUser,
) -> impl Future<Item = Result<User, Error>, Error = BlockingError> {
    repo.run(move |conn| {
        // TODO: store password not in plain text, later
        diesel::insert_into(users::table)
            .values(&user)
            .get_result(&conn)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::generate;
    use fake::fake;
    use tokio_async_await_test::async_test;

    #[async_test]
    async fn test_create_user() {
        let repo = Repo::new();

        let new_user = generate::new_user();
        let user = await! { insert(repo.clone(), new_user) }.expect("Create user failed.");

        let results = await! {
           find(repo.clone(), user.id)
        };
        assert!(results.is_ok());
    }
}