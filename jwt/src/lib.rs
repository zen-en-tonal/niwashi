mod errors;

use errors::Error;
use jsonwebtoken::{decode, encode, Header, Validation};
use life::prelude::Mutator;
use serde::{Deserialize, Serialize};

pub use jsonwebtoken::{DecodingKey, EncodingKey};
pub use life::prelude::{MakeIntrospective, MakeStopGrowing, Mutable, Validator};

pub fn create<T: Serialize + Default>(key: &EncodingKey) -> Result<String, Error> {
    Ok(encode(&Header::default(), &T::default(), key)?)
}

pub fn create_with<T: Serialize>(
    factory: impl Fn() -> T,
    key: &EncodingKey,
) -> Result<String, Error> {
    Ok(encode(&Header::default(), &factory(), key)?)
}

pub fn update<T>(
    token: &impl AsRef<str>,
    encoding_key: &EncodingKey,
    decoding_key: &DecodingKey,
    validator: impl Validator<T>,
) -> Result<String, Error>
where
    T: for<'de> Deserialize<'de> + Serialize + Mutable + AsRef<T>,
{
    let token = decode::<T>(token.as_ref(), decoding_key, &Validation::default())?;
    match validator.make_introspective(token.claims).mutate() {
        Some(some) => Ok(encode(&token.header, &some.inner(), encoding_key)?),
        None => Err(Error::InvalidToken),
    }
}

pub fn update_with<T>(
    token: &impl AsRef<str>,
    mutator: impl Mutator<T>,
    encoding_key: &EncodingKey,
    decoding_key: &DecodingKey,
    validator: impl Validator<T>,
) -> Result<String, Error>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let token = decode::<T>(token.as_ref(), decoding_key, &Validation::default())?;
    let mutable = validator.make_introspective(mutator.make_mutable(token.claims));
    match mutable.mutate() {
        Some(some) => Ok(encode(&token.header, some.inner(), encoding_key)?),
        None => Err(Error::InvalidToken),
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
        let new_token = update::<Claims>(&old_token, &e_key, &d_key, |_: &Claims| true).unwrap();
        assert_ne!(old_token, new_token)
    }
}
