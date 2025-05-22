# Web Engineering Project

## Tech Stack
- backend: Rust + Rocket
- frontend: Svelte
- db: SurrealDB

## API Testing
The Webeng.json file can be imported into hoppscotch (and Postman?)
and contains the API endpoints with examples for the project.

## Running
To run the app
### Set up the environment
Fill out the env values. See .env.example for the required values (you can make
up a `SURREALDB_USER` and `SURREALDB_PASS`).
```sh
cp .env.example .env
```
If you intend to run the app in release mode, `ROCKET_SECRET_KEY` must be set.
This file does not need to be sourced, it's presence suffices.

### Start the database
```sh
docker compose up
```
### Start the server:
```sh
cargo run # for debug builds
cargo run --release # for optimized builds
cargo run -F fingerprinting # to include the fingerprinting mechanism
```
## Docs
The documentation can be automatically generated:
```sh
cargo doc --no-deps
```

### State

The webserver is completely stateless as it saves all persistent state in
a SurrealDB cloud instance. This means it can easily be scaled using a load
balancer.

> [!warning] File Upload
> This is correct at the time of writing. But we do not have image uploads yet.
> For images we may want to use the S3 free tier.

> [!warning] DDoS Protection
> We want to have rate-limiting while the server can remain stateless.
> It is not yet implemented but being worked on.
