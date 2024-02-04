use session::*;
use uuid::Uuid;

struct PgSessions;

struct NullValidation;

impl Validation for NullValidation {
    fn is_valid(&self, session: &Session) -> bool {
        true
    }
}

impl GetSession<PgMutateSession, NullValidation> for PgSessions {
    type Token = Uuid;

    fn get(
        &self,
        token: &Self::Token,
    ) -> Result<session::MutableSession<PgMutateSession, NullValidation>, GetSessionError> {
        todo!()
    }
}

struct PgMutateSession(Uuid);

impl Mutation for PgMutateSession {
    type Token = Uuid;

    fn update(self) -> Result<Self::Token, session::MutationError> {
        todo!()
    }

    fn destroy(self) -> Result<(), session::MutationError> {
        todo!()
    }
}

fn f() {
    let r = update_session(PgSessions, Uuid::new_v4());
}
