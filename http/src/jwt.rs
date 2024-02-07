use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::Error;

pub fn encode<T: Serialize>(encoding_key: EncodingKey) -> impl Fn(T) -> Result<String, Error> {
    move |x: T| {
        Ok(
            jsonwebtoken::encode::<T>(&Header::default(), &x, &encoding_key)
                .map_err(|e| Error::Encode(Box::new(e)))?,
        )
    }
}

pub fn decode<T: for<'de> Deserialize<'de>>(
    decoding_key: DecodingKey,
) -> impl Fn(String) -> Result<T, Error> {
    move |x: String| {
        Ok(
            jsonwebtoken::decode::<T>(&x, &decoding_key, &Validation::default())
                .map_err(|e| Error::Encode(Box::new(e)))
                .map(|x| x.claims)?,
        )
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Claims {
    exp: i64,
    from: i64,
    count: i64,
}

pub(crate) fn factory(now: i64, exp_in: i64) -> impl Fn() -> Result<Claims, Error> {
    move || {
        Ok(Claims {
            exp: now + exp_in,
            from: now,
            count: 1,
        })
    }
}

pub(crate) fn mutator(now: i64, exp_in: i64) -> impl Fn(Claims) -> Claims + Clone {
    move |jwt: Claims| Claims {
        exp: now + exp_in,
        from: jwt.from,
        count: jwt.count + 1,
    }
}

pub(crate) fn validator(now: i64, min: f64, max: f64) -> impl Fn(&Claims) -> bool + Clone {
    move |jwt: &Claims| {
        let duration = now - jwt.from;
        let density = match (jwt.count, duration) {
            (0, 0) => 0.0,
            (x, 0) => x as f64,
            (x, d) => x as f64 / d as f64,
        };
        min <= density && density <= max
    }
}

pub(crate) fn destroy(_: &Claims) {}
