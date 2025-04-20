# Web Engineering Project

## Tech Stack
- backend: Rust + Rocket
- frontend: n/a
- db: SurrealDB

## API Testing
The Webeng.json file can be imported into hoppscotch (and Postman?)
and contains the API endpoints with examples for the project.

## Notes

### env
To run this application, you need to set up a valid `.env` file.
`.env.example` should show which options need to be set. Ask for the correct
values if you are not sure.
This file does not need to be sourced, it's presence suffices.

### State

The webserver is completely stateless as it saves all persistent state in
a SurrealDB cloud instance. This means it can easily be scaled using a load
balancer.

> [!warning] File Upload
> This is correct at the time of writing. But we do not have image uploads yet.
> For images we may want to use the S3 free tier.
