//! This module includes fingerprinting functionality, meaning it enables us to
//! distinguish clients based on a few parameters.
//!  At the moment, these parameters are
//! - user_agent
//! - client_ip
//! - session_id Cookie
//!
//! more could be added with relative ease in the future.
//! # Embedding
//! We use an embeddings model to vectorize said parameters, storing the points
//! in a vector space, where we can query for the closest neighbours, assuming
//! points with a distance under a certain threshold to be the same client.
//! # Performance
//! Generating embeddings for every request is very resource intensive, and
//! not worth the advantage it gives. Frankly it's an overengineered, mostly
//! useless when factoring in its cost piece of code that could be implemented
//! in a simpler fashion. But it could be sped up significantly by using a
//! custom made vectorisation primitive
use std::{iter::zip, net::IpAddr};

use crate::dbg_print;
use fastembed::{InitOptions, TextEmbedding};
use rocket::{
    Request, State,
    fairing::{Fairing, Kind},
    http::{CookieJar, HeaderMap},
    outcome::Outcome,
    request::{self, FromRequest},
};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, Surreal, engine::any::Any};

/// The maximum manhattan distance two points in vector space may be apart to
/// still be considered the same
pub static VECTOR_DISTANCE_THRESHOLD: f32 = 0.01;

/// A rocket middleware that takes parameters from the request like the
/// session_id cookie, the user agent, the remote ip and computes an embedding.
/// It is ran as a [Fairing], hooking into the request cycle
pub struct Fingerprinter;

/// [NnSearchResult] is one of the results of the query defined in
/// src/queries/check_nearest_fingerprint.
/// It comes in two flavours: `found_before` and `created` being set,
/// or `found_before` and `nearest_neighbour` being set.
/// # Note
/// Having this configuration with mutual exclusivity is bad, as it enables
/// invalid states (both are unset or both are set) and we lose out on the
/// compilers strict type checking. Ideally we would want to
/// make this two separate structs (and maybe combine them in an enum), but this
/// makes deserialising the database result more verbose. So we leave it like
/// this for now
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NnSearchResult {
    /// When found_before is true, that means the database has identified
    /// another point within the threshold defined by
    /// [VECTOR_DISTANCE_THRESHOLD]. This also means that `nearest_neighbour` is
    /// set.
    /// When it is false, no such point could be identified. The query then
    /// creates a new entry, whose ID it returns in `created`
    found_before: bool,
    /// The id of the newly created point.
    created: Option<RecordId>,
    /// The id of the newly created neighbour.
    nearest_neighbour: Option<RecordId>,
}

/// This is called once at the beginning of the main method. It instantiates a
/// text embeddings model which is then attached to the rocket server.
pub fn init_embeddings_model() -> Option<TextEmbedding> {
    TextEmbedding::try_new(InitOptions::default()).ok()
}

/// The [Fingerprinter] is a module of the server that acts as a middleware,
/// calculating a clients fingerprint for every request.
/// It then sends the fingerprint, which is a 384-dimensional f32 vector to the
/// database in order to group requests for rate-limiting
impl<'r> Fingerprinter {
    fn generate_embeddings(
        model: &State<TextEmbedding>,
        user_agent: Option<String>,
        source_ip: Option<String>,
        session_id: Option<String>,
    ) -> Option<Vec<f32>> {
        let documents = vec![format!(
            "user_agent: {}, source_ip: {}, session_id: {}",
            user_agent.unwrap_or("No User-Agent".into()),
            source_ip.unwrap_or("No Source IP".into()),
            session_id.unwrap_or("No Cookie".into())
        )];

        // Generate embeddings with the default batch size, 256
        Some(model.embed(documents, None).ok()?.get(0)?.to_owned())
    }
    async fn track_request(
        model: &State<TextEmbedding>,
        db: &State<Surreal<Any>>,
        tracker: TrackingInfo<'r>,
    ) -> Option<NnSearchResult> {
        let session_id = tracker
            .cookies
            .and_then(|c| c.get("session_id").map(|c| c.value().to_string()));
        let user_agent = tracker.headers.get_one("User-Agent").map(|h| h.to_string());
        let source_ip = tracker.client_ip.map(|ip| ip.to_string());
        dbg_print!(&session_id, &user_agent, &source_ip);
        let embed = Fingerprinter::generate_embeddings(
            model,
            user_agent.clone(),
            source_ip.clone(),
            session_id.clone(),
        );
        dbg_print!(&embed);
        let query = db
            .query(include_str!("queries/check_nearest_fingerprint.surql"))
            .bind(("MAX_DIST", VECTOR_DISTANCE_THRESHOLD))
            .bind(("embeddings", embed.unwrap()))
            .bind(("user_agent", user_agent.unwrap_or("No user agent".into())))
            .bind(("source_ip", source_ip.unwrap_or("No source IP".into())))
            .bind((
                "session_id",
                session_id.unwrap_or("No session_id cookie".into()),
            ))
            .await;
        dbg_print!(&query);
        let mut res = match query {
            Err(_) => {
                return None;
            }
            Ok(res) => res,
        };
        let returned = match res.take::<Vec<NnSearchResult>>(2) {
            Err(_) => {
                return None;
            }
            Ok(val) => match val.get(0) {
                None => {
                    return None;
                }
                Some(v) => v.clone(),
            },
        };
        dbg_print!(&returned);
        Some(returned)
    }
}

/// Calculates the L1 distance between two vectors
pub fn manhattan_dist(v1: Vec<f32>, v2: Vec<f32>) -> f32 {
    let mut sum = 0f32;
    for (u, v) in zip(v1, v2) {
        sum += (u - v).abs()
    }
    sum
}

#[rocket::async_trait]
impl Fairing for Fingerprinter {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Fingerprint",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut rocket::Data<'_>) {
        if let Some(route) = request.route() {
            dbg_print!(route.uri.path());
            if !route.uri.path().starts_with("/trackme") {
                return;
            }
        } else {
            // if we can not get the route, we should not perform
            // fingerprinting, as it is quite compute intensive
            return;
        }
    }

    async fn on_response<'r>(
        &self,
        _request: &'r Request<'_>,
        response: &mut rocket::Response<'r>,
    ) {
        // Don't change a successful user's response, ever.
        if response.status() != rocket::http::Status::NotFound {
            return;
        }
    }
}
#[rocket::get("/trackme")]
pub async fn route_trackme<'r>(
    model: &State<TextEmbedding>,
    db: &State<Surreal<Any>>,
    tracker: TrackingInfo<'r>,
) -> String {
    format!(
        "{:?}",
        Fingerprinter::track_request(model, db, tracker).await
    )
}

#[rocket::get("/trackme")]
pub async fn route_frontend_trackme<'r>(
    model: &State<TextEmbedding>,
    db: &State<Surreal<Any>>,
    tracker: TrackingInfo<'r>,
) -> Template {
    let fingerprint = Fingerprinter::track_request(model, db, tracker)
        .await
        .unwrap();
    let nn = fingerprint
        .nearest_neighbour
        .map(|v| v.key().to_string())
        .unwrap_or("".to_string());
    // let nn = {
    //     if let Some(nn) = fingerprint.nearest_neighbour {
    //         nn.to_string()
    //     } else {
    //         "".to_string()
    //     }
    let fb = fingerprint.found_before;
    let ni = fingerprint
        .created
        .map(|v| v.key().to_string())
        .unwrap_or("".to_string());
    Template::render(
        "trackme",
        context! {
            nn: nn,
            found_before: fb,
            new_key: ni,
        },
    )
}

/// [TrackingInfo] contains information we use to fingerprint clients.
/// It can be used as a request guard
pub struct TrackingInfo<'r> {
    pub headers: HeaderMap<'r>,
    pub client_ip: Option<IpAddr>,
    pub cookies: Option<CookieJar<'r>>,
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for TrackingInfo<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, ()> {
        let headers = request.headers().to_owned();
        let cookies = match request.guard::<&CookieJar<'r>>().await {
            Outcome::Success(c) => Some(c.to_owned()),
            _ => None,
        };
        let client_ip = request.client_ip();
        return Outcome::Success(TrackingInfo {
            headers,
            client_ip,
            cookies,
        });
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn get_fingerprints() {
        //! This function is not actually a test, I just use it to generate
        //! fingerprints for testing
        let model = init_embeddings_model().unwrap();
        let user_agent = Some(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/48.0.2035.1237 Safari/537.36".into(),
        );
        let source_ip = Some("107.2.95.249".into());
        let session_id = Some("4d593b5c-646e-484d-b434-d38f7b853987".into());
        let embed = Fingerprinter::generate_embeddings(
            &State::from(&model),
            user_agent.clone(),
            source_ip.clone(),
            session_id.clone(),
        );
        println!(
            "{:?}\n{:?}\n{:?}\n{:?}",
            source_ip, session_id, user_agent, embed
        );
    }
}
