use session::*;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::runtime::Runtime;
pub use uuid::Uuid;

pub type SessionId = Uuid;

pub struct PgSessions(Pool<Postgres>);

impl PgSessions {
    pub async fn new(url: impl AsRef<str>) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(url.as_ref())
            .await?;
        Ok(Self(pool))
    }
}

struct NullValidation;

impl Validation for NullValidation {
    fn is_valid(&self, _: &Session) -> bool {
        true
    }
}

impl GetSession<PgMutateSession, NullValidation> for PgSessions {
    type Token = SessionId;

    fn get(
        &self,
        token: &Self::Token,
    ) -> Result<session::MutableSession<PgMutateSession, NullValidation>, GetSessionError> {
        #[derive(sqlx::FromRow)]
        struct PgSession {
            from: Option<DateTime>,
            latest: Option<DateTime>,
            len: Option<i64>,
        }
        let rt = Runtime::new().unwrap();
        let res = rt
            .block_on(
                sqlx::query_as!(
                    PgSession,
                    "
                    SELECT
                        MIN(created_at) AS from
                    ,   MAX(created_at) AS latest
                    ,   COUNT(session_id) AS len
                    FROM
                        sessions
                    WHERE
                        session_chain_id = (
                            SELECT
                                session_chain_id
                            FROM
                                sessions
                            WHERE
                                session_id = $1
                        )
                    ",
                    token
                )
                .fetch_one(&self.0),
            )
            .map_err(|e| GetSessionError::Other(Box::new(e)))?;
        if res.len.unwrap_or_default() < 1 {
            return Err(GetSessionError::InvalidToken);
        }
        let session = Session::new(
            res.from.unwrap(),
            res.latest.unwrap(),
            res.len.unwrap() as usize,
        );
        Ok(MutableSession::new(
            PgMutateSession(self.0.clone(), token.clone()),
            NullValidation,
            session,
        ))
    }
}

struct PgMutateSession(Pool<Postgres>, SessionId);

impl Mutation for PgMutateSession {
    type Token = SessionId;

    fn update(self) -> Result<Self::Token, session::MutationError> {
        let new_token = Uuid::new_v4();
        let rt = Runtime::new().unwrap();
        rt.block_on(
            sqlx::query!(
                "
                INSERT INTO sessions (session_id, session_chain_id, created_at)
                VALUES (
                    $1,
                    (
                        SELECT
                            session_chain_id
                        FROM
                            sessions
                        WHERE
                            session_id = $2
                    ),
                    $3
                )
                ",
                new_token,
                self.1,
                now()
            )
            .execute(&self.0),
        )
        .map_err(|e| MutationError::Other(Box::new(e)))?;
        Ok(new_token)
    }

    fn destroy(self) -> Result<(), session::MutationError> {
        let rt = Runtime::new().unwrap();
        rt.block_on(
            sqlx::query!(
                "
                DELETE FROM sessions
                WHERE
                    session_chain_id = (
                        SELECT
                            session_chain_id
                        FROM
                            sessions
                        WHERE
                            session_id = $1
                    )
                ",
                self.1
            )
            .fetch_one(&self.0),
        )
        .map_err(|e| MutationError::Other(Box::new(e)))?;
        Ok(())
    }
}
