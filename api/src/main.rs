use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use jwt::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};

fn now() -> DateTime<Utc> {
    chrono::Utc::now()
}

struct AppState {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    min_density: f64,
    max_density: f64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let key = AppState {
        encoding_key: EncodingKey::from_secret(b""),
        decoding_key: DecodingKey::from_secret(b""),
        min_density: 0.1,
        max_density: 2.0,
    };
    let state = Arc::new(key);

    let app = Router::new()
        .route("/health", get(health))
        .route("/new", post(create))
        .route("/update", post(update))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn create(State(s): State<Arc<AppState>>) -> (StatusCode, Json<TokenPayload>) {
    match jwt::create::<Claims>(&s.encoding_key) {
        Ok(token) => (
            StatusCode::OK,
            Json(TokenPayload {
                is_valid: true,
                token: Some(token),
                err: None,
            }),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TokenPayload {
                is_valid: false,
                token: None,
                err: Some(err.to_string()),
            }),
        ),
    }
}

#[derive(Deserialize)]
struct UpdateQuery {
    token: String,
}

async fn update(
    State(s): State<Arc<AppState>>,
    Query(q): Query<UpdateQuery>,
) -> (StatusCode, Json<TokenPayload>) {
    let res = jwt::update::<Claims>(&q.token, &s.encoding_key, &s.decoding_key)
        .introspective(validate(now(), s.min_density, s.max_density));
    match res {
        Ok(token) => (
            StatusCode::OK,
            Json(TokenPayload {
                is_valid: true,
                token: Some(token),
                err: None,
            }),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TokenPayload {
                is_valid: false,
                token: None,
                err: Some(err.to_string()),
            }),
        ),
    }
}

#[derive(Debug, Serialize)]
struct TokenPayload {
    is_valid: bool,
    token: Option<String>,
    err: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    exp: i64,
    from: i64,
    count: i64,
}

fn validate(n: DateTime<Utc>, min_density: f64, max_density: f64) -> impl Fn(&Claims) -> bool {
    move |claims: &Claims| {
        let duration = n - DateTime::from_timestamp(claims.from, 0).unwrap();
        let density = match (claims.count, duration.num_seconds()) {
            (0, 0) => 0f64,
            (x, 0) => x as f64,
            (x, d) => x as f64 / d as f64,
        };
        min_density <= density && density <= max_density
    }
}

impl jwt::Mutable for Claims {
    fn mutate(self) -> Option<Self> {
        let exp = now() + Duration::hours(1);
        Some(Claims {
            exp: exp.timestamp(),
            from: self.from,
            count: self.count + 1,
        })
    }
}

impl Default for Claims {
    fn default() -> Self {
        let from = now();
        let exp = from + Duration::hours(1);
        Self {
            exp: exp.timestamp(),
            from: from.timestamp(),
            count: 1,
        }
    }
}

impl AsRef<Claims> for Claims {
    fn as_ref(&self) -> &Claims {
        &self
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::{validate, Claims};

    #[test]
    fn density() {
        let c = Claims {
            exp: 1337,
            from: 1337,
            count: 1,
        };
        let invalid = validate(DateTime::from_timestamp(1337 + 1, 0).unwrap(), 0f64, 1f64)(&c);
        assert_eq!(invalid, false)
    }
}
