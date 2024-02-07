#[cfg(feature = "jwt")]
mod jwt;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use life::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::{fmt::Display, marker::PhantomData, sync::Arc};

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

    let expires_in = env::var("EXPIRES_IN")
        .unwrap_or("60".to_string())
        .parse::<i64>()
        .unwrap();
    let min_density = env::var("MIN_DENSITY")
        .unwrap_or("0.0".to_string())
        .parse::<f64>()
        .unwrap();
    let max_density = env::var("MAX_DENSITY")
        .unwrap_or("5.0".to_string())
        .parse::<f64>()
        .unwrap();
    let secret = env::var("SECRET").unwrap();
    let port = env::var("PORT")
        .unwrap_or("3000".to_string())
        .parse::<i32>()
        .unwrap();

    #[cfg(feature = "jwt")]
    let state = AppState {
        factory: jwt::factory(now, chrono::Duration::minutes(expires_in).num_seconds()),
        mutator: jwt::mutator(now, chrono::Duration::minutes(expires_in).num_seconds()),
        validator: jwt::validator(now, min_density, max_density),
        destroyer: jwt::destroy,
        encode: jwt::encode(jsonwebtoken::EncodingKey::from_secret(secret.as_bytes())),
        decode: jwt::decode(jsonwebtoken::DecodingKey::from_secret(secret.as_bytes())),
        marker: PhantomData,
    };

    let state = Arc::new(state);

    let app = Router::new()
        .route("/health", get(health))
        .route("/new", post(create))
        .route("/update", post(update))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug)]
enum Error {
    #[allow(dead_code)]
    Create,
    InvalidToken,
    Encode(Box<dyn std::error::Error>),
    #[allow(dead_code)]
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
