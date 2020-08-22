use diesel_migrations as dm;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyrHubstorageError {
    #[error(transparent)]
    RunMigrations(#[from] dm::RunMigrationsError),
    #[error(transparent)]
    DieselError(#[from] diesel::result::Error),
    #[error("Not a valid token")]
    InvalidToken,
    #[error("Unknown user")]
    UnknownUser,
    #[error("Nickname {0} is already being used")]
    AlreadyUsedNickname(String),
    #[error("User is frozen")]
    FrozenUser,
}

pub type Result<R> = std::result::Result<R, KeyrHubstorageError>;
