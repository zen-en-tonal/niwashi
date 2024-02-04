pub use chrono::Utc;

pub type DateTime = chrono::DateTime<Utc>;
pub type Duration = chrono::Duration;

pub fn now() -> DateTime {
    chrono::Utc::now()
}

use std::{error::Error, fmt::Display};

pub struct Session {
    from: DateTime,
    latest: DateTime,
    len: usize,
}

impl Session {
    pub fn new(from: DateTime, latest: DateTime, len: usize) -> Self {
        Self { from, latest, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn duration(&self) -> Duration {
        self.latest - self.from
    }

    pub fn density_min(&self) -> f32 {
        match (self.len(), self.duration().num_minutes()) {
            (0, 0) => 0.0,
            (_, 0) => f32::MAX,
            (len, d) => len as f32 / d as f32,
        }
    }

    pub fn from(&self) -> DateTime {
        self.from
    }

    pub fn latest(&self) -> DateTime {
        self.latest
    }
}

pub struct MutableSession<M, V> {
    mutation: M,
    validation: V,
    session: Session,
}

pub trait Mutation {
    type Token;
    fn update(self) -> Result<Self::Token, MutationError>;
    fn destroy(self) -> Result<(), MutationError>;
}

pub enum MutationError {
    InvalidSession,
    UpdateFailure,
    DestroyFailure,
    Other(Box<dyn Error>),
}

impl Display for MutationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub trait Validation {
    fn is_valid(&self, session: &Session) -> bool;
}

impl<M, V> MutableSession<M, V>
where
    M: Mutation,
    V: Validation,
{
    pub fn new(mutation: M, validation: V, session: Session) -> Self {
        Self {
            mutation,
            validation,
            session,
        }
    }

    fn update(self) -> Result<M::Token, MutationError> {
        if !self.validation.is_valid(&self.session) {
            self.mutation.destroy()?;
            return Err(MutationError::InvalidSession);
        }
        self.mutation.update()
    }
}

impl<M, V> MutableSession<M, V>
where
    M: Mutation,
{
    fn destroy(self) -> Result<(), MutationError> {
        self.mutation.destroy()
    }
}

pub trait GetSession<M, V> {
    type Token;
    fn get(&self, token: &Self::Token) -> Result<MutableSession<M, V>, GetSessionError>;
}

pub enum GetSessionError {
    InvalidToken,
    Other(Box<dyn Error>),
}

pub fn update_session<S, T, M, V>(sessions: S, token: T) -> Result<T, SessionError>
where
    S: GetSession<M, V, Token = T>,
    M: Mutation<Token = T>,
    V: Validation,
{
    let sessions = sessions.get(&token)?;
    Ok(sessions.update()?)
}

pub fn destroy_session<S, T, M, V>(sessions: S, token: T) -> Result<(), SessionError>
where
    S: GetSession<M, V, Token = T>,
    M: Mutation<Token = T>,
{
    let sessions = sessions.get(&token)?;
    Ok(sessions.destroy()?)
}

pub enum SessionError {
    Repository(GetSessionError),
    Mutation(MutationError),
}

impl From<GetSessionError> for SessionError {
    fn from(value: GetSessionError) -> Self {
        SessionError::Repository(value)
    }
}

impl From<MutationError> for SessionError {
    fn from(value: MutationError) -> Self {
        SessionError::Mutation(value)
    }
}
