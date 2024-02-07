#[cfg(feature = "jwt")]
mod jwt;

use std::{fmt::Display, marker::PhantomData, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use life::prelude::*;
use serde::{Deserialize, Serialize};

struct AppState<Factory, Mutation, Validation, Destroy, Encode, Decode, T> {
    factory: Factory,
    mutator: Mutation,
    validator: Validation,
    destroyer: Destroy,
    encode: Encode,
    decode: Decode,
    marker: PhantomData<T>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    fn now() -> i64 {
        chrono::Utc::now().timestamp()
    }

    #[cfg(feature = "jwt")]
    let state = AppState {
        factory: jwt::factory(now(), chrono::Duration::hours(1).num_seconds()),
        mutator: jwt::mutator(now(), chrono::Duration::hours(1).num_seconds()),
        validator: jwt::validator(now(), 0.1, 2.0),
        destroyer: jwt::destroy,
        encode: jwt::encode(jsonwebtoken::EncodingKey::from_secret(b"")),
        decode: jwt::decode(jsonwebtoken::DecodingKey::from_secret(b"")),
        marker: PhantomData,
    };

    let state = Arc::new(state);

    let app = Router::new()
        .route("/health", get(health))
        .route("/new", post(create))
        .route("/update", post(update))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug)]
enum Error {
    Create,
    InvalidToken,
    Encode(Box<dyn std::error::Error>),
    Decode(Box<dyn std::error::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Create => f.write_str("faild to initalize token"),
            Error::InvalidToken => f.write_str("invalid token"),
            Error::Encode(e) => e.fmt(f),
            Error::Decode(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn create<Factory, Mutation, Validation, Destroy, Encode, Decode, T>(
    State(s): State<Arc<AppState<Factory, Mutation, Validation, Destroy, Encode, Decode, T>>>,
) -> (StatusCode, Json<TokenResp>)
where
    Factory: Fn() -> Result<T, Error>,
    Encode: Fn(T) -> Result<String, Error>,
{
    match (s.factory)() {
        Ok(token) => match (s.encode)(token) {
            Ok(token) => (StatusCode::OK, Json(TokenResp::ok(token))),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(TokenResp::err(err))),
        },
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(TokenResp::err(err))),
    }
}

#[derive(Deserialize)]
struct UpdateQuery {
    token: String,
}

async fn update<Factory, Mutation, Validation, Destroy, Encode, Decode, T>(
    State(s): State<Arc<AppState<Factory, Mutation, Validation, Destroy, Encode, Decode, T>>>,
    Query(q): Query<UpdateQuery>,
) -> (StatusCode, Json<TokenResp>)
where
    Mutation: Mutator<T> + Clone,
    Validation: Validator<T> + Clone,
    Destroy: Destroyer<T> + Clone,
    Encode: Fn(T) -> Result<String, Error>,
    Decode: Fn(String) -> Result<T, Error>,
    T: Clone,
{
    match (s.decode)(q.token) {
        Ok(token) => {
            let mutator = s
                .destroyer
                .clone()
                .make_fragile(s.validator.clone(), s.mutator.clone().make_mutable(token));
            match mutator.mutate() {
                Some(next) => match (s.encode)(next.inner().clone()) {
                    Ok(token) => (StatusCode::OK, Json(TokenResp::ok(token))),
                    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(TokenResp::err(err))),
                },
                None => (
                    StatusCode::FORBIDDEN,
                    Json(TokenResp::err(Error::InvalidToken)),
                ),
            }
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(TokenResp::err(err))),
    }
}

#[derive(Debug, Serialize)]
struct TokenResp {
    is_valid: bool,
    token: Option<String>,
    err: Option<String>,
}

impl TokenResp {
    fn ok(token: String) -> TokenResp {
        TokenResp {
            is_valid: true,
            token: Some(token),
            err: None,
        }
    }

    fn err(err: impl std::error::Error) -> TokenResp {
        TokenResp {
            is_valid: false,
            token: None,
            err: Some(err.to_string()),
        }
    }
}
