# Getting started

Ensure postgres is installed and running.
Ensure user 'realworld-gotham' exists and can create databases.
```
sudo -u postgres psql -c "CREATE USER \"realworld-gotham\" WITH ENCRYPTED PASSWORD 'password';"
sudo -u postgres psql -c "ALTER USER \"realworld-gotham\" CREATEDB;"
```
Ensure diesel cli is installed, see [http://diesel.rs/guides/getting-started/]
