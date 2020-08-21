use thiserror::Error;
use diesel_migrations as dm;

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
}

pub type Result<R> = std::result::Result<R, KeyrHubstorageError>;
