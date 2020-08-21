/* keyr -- keep track of your keystrokes
 * Copyright (c) 2020 Thomas Letan
 *
 * This file is part of keyr.
 *
 * keyr is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * keyr is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with keyr.  If not, see <https://www.gnu.org/licenses/>.
 */

use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use thiserror::Error;

use keyr_hubstorage as kbs;
use kbs::error::KeyrHubstorageError;

#[derive(Error, Debug)]
pub enum KeyrHubError {
    #[error("Client trying to access a protected route without setting Keyr-Token header")]
    MissingKeyrTokenHeader,
    #[error(transparent)]
    Storage(#[from] KeyrHubstorageError),
    #[error(transparent)]
    Pool(#[from] r2d2::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("The requested data are not public")]
    PrivateData
}

impl From<diesel::result::Error> for KeyrHubError {
    fn from(err : diesel::result::Error) -> KeyrHubError {
        KeyrHubstorageError::from(err).into()
    }
}

pub type Result<R> = std::result::Result<R, KeyrHubError>;

impl ResponseError for KeyrHubError {
    fn status_code(&self) -> StatusCode {
        match self {
            KeyrHubError::PrivateData =>
                StatusCode::UNAUTHORIZED,
            KeyrHubError::MissingKeyrTokenHeader => StatusCode::UNAUTHORIZED,
            KeyrHubError::Storage(KeyrHubstorageError::InvalidToken) =>
                StatusCode::UNAUTHORIZED,
            KeyrHubError::Storage(KeyrHubstorageError::UnknownUser) =>
                StatusCode::BAD_REQUEST,
            KeyrHubError::Storage(KeyrHubstorageError::AlreadyUsedNickname(_)) =>
                StatusCode::BAD_REQUEST,
            KeyrHubError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            KeyrHubError::Pool(_) => StatusCode::INTERNAL_SERVER_ERROR,
            KeyrHubError::IO(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
