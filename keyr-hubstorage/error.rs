use thiserror::Error;
use diesel_migrations as dm;

#[derive(Error, Debug)]
pub enum KeyrHubstorageError {
    #[error(transparent)]
    RunMigrations(#[from] dm::RunMigrationsError),
    #[error(transparent)]
    DieselError(#[from] diesel::result::Error),
}

pub type Result<R> = std::result::Result<R, KeyrHubstorageError>;
