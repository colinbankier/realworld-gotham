# ![RealWorld Example App](logo.png)

> ### Rust/Gotham codebase containing real world examples (CRUD, auth, advanced patterns, etc) that adheres to the [RealWorld](https://github.com/gothinkster/realworld) spec and API.


### [Demo](https://github.com/gothinkster/realworld)&nbsp;&nbsp;&nbsp;&nbsp;[RealWorld](https://github.com/gothinkster/realworld)

### WIP - this repo is as yet incomplete and still being implemented

This codebase was created to demonstrate a fully fledged fullstack application built with **Rust/Gotham** including CRUD operations, authentication, routing, pagination, and more.

This project attempts to acheive the following:
 - Separate domain logic from web logic. The `conduit` module contains domain logic and the `web` module has logic for dealing with http stuff and json request/response formats.
 - Async queries with diesel. Diesel doesn't directly support async, but we can still build an async application around it using `tokio_threadpool::blocking`. The `db` module provides a `Repo` abstraction to encapsulate this.
 - Parallel database tests. Tests use isolated test transactions so database tests can be run in parallel.

For more information on how to this works with other frontends/backends, head over to the [RealWorld](https://github.com/gothinkster/realworld) repo.

# Getting started

Ensure postgres is installed and running.
Ensure user 'realworld-gotham' exists and can create databases.
```
sudo -u postgres psql -c "CREATE USER \"realworld-gotham\" WITH ENCRYPTED PASSWORD 'password';"
sudo -u postgres psql -c "ALTER USER \"realworld-gotham\" CREATEDB;"
```
Ensure diesel cli is installed, see [http://diesel.rs/guides/getting-started/]

## Run tests
Run tests, including DB integration tests
```
cargo make test
```

## Run the app
Setup database using diesel cli
```
diesel database setup
```
Run the app
```
cargo run
```