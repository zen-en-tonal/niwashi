mod errors;

use std::marker::PhantomData;

use errors::Error;
use jsonwebtoken::{decode, encode, Header, Validation};
use serde::{Deserialize, Serialize};

pub use jsonwebtoken::{DecodingKey, EncodingKey};
pub use life::prelude::{MakeIntrospective, MakeStopGrowing, Mutable, Validator};

pub fn create<T: Serialize + Default>(key: &EncodingKey) -> Result<String, Error> {
    Ok(encode(&Header::default(), &T::default(), key)?)
}

pub fn update<'a, T>(
    token: &'a impl AsRef<str>,
    encoding_key: &'a EncodingKey,
    decoding_key: &'a DecodingKey,
) -> Update<'a, T> {
    Update {
        token: token.as_ref(),
        encoding_key,
        decoding_key,
        marker: PhantomData,
    }
}

pub struct Update<'a, T> {
    token: &'a str,
    encoding_key: &'a EncodingKey,
    decoding_key: &'a DecodingKey,
    marker: PhantomData<T>,
}

impl<'a, T> Update<'a, T>
where
    T: Serialize + for<'de> Deserialize<'de> + Mutable + AsRef<T>,
{
    pub fn introspective(&self, validator: impl Validator<T>) -> Result<String, Error> {
        self.update(|c| validator.make_introspective(c))
    }

    pub fn anyway(&self) -> Result<String, Error> {
        self.update(|c| c)
    }

    fn update<F: FnOnce(T) -> M, M: Mutable + AsRef<T>>(&self, m: F) -> Result<String, Error> {
        let claims = decode::<T>(self.token, self.decoding_key, &Validation::default())?;
        let mutable = m(claims.claims);
        match mutable.mutate() {
            Some(x) => Ok(encode(&claims.header, &x.as_ref(), self.encoding_key)?),
            None => Err(Error::InvalidToken),
        }
    }
}

impl<'a, T> Update<'a, T>
where
    T: Serialize + for<'de> Deserialize<'de> + Mutable + AsRef<T> + Clone,
{
    pub fn stop_growing(&self, validator: impl Validator<T>) -> Result<String, Error> {
        self.update(|c| validator.make_stop_growing(c))
    }
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::{DecodingKey, EncodingKey};
    use life::prelude::Mutable;
    use serde::{Deserialize, Serialize};

    use crate::{create, update};

    #[derive(Clone, Serialize, Deserialize)]
    struct Claims {
        exp: usize,
        field: i32,
    }

    impl AsRef<Claims> for Claims {
        fn as_ref(&self) -> &Claims {
            self
        }
    }

    impl Default for Claims {
        fn default() -> Self {
            Self {
                exp: usize::MAX,
                field: Default::default(),
            }
        }
    }

    impl Mutable for Claims {
        fn mutate(self) -> Option<Self> {
            Some(Claims {
                exp: usize::MAX,
                field: self.field + 1,
            })
        }
    }

    #[test]
    fn create_to_update() {
        let e_key = EncodingKey::from_secret("secret".as_ref());
        let d_key = DecodingKey::from_secret("secret".as_ref());
        let old_token = create::<Claims>(&e_key).unwrap();
        let new_token = update::<Claims>(&old_token, &e_key, &d_key)
            .anyway()
            .unwrap();
        assert_ne!(old_token, new_token)
    }
}
