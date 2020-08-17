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

use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use futures::future::{Ready, ok, err};

use keyr_hubstorage as kbs;
use kbs::users::Token;

use crate::error::KeyrHubError;

pub struct TokenHeader(Token);

impl TokenHeader {
    pub fn as_token(&self) -> &Token {
        &self.0
    }
}

impl FromRequest for TokenHeader {
    type Config = ();
    type Error = KeyrHubError;
    type Future = Ready<Result<TokenHeader, KeyrHubError>>;

    fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
        if let Some(token) = req.headers().get("keyr-token") {
            if let Ok(token) = token.to_str() {
                ok(TokenHeader(Token(token.to_owned())))
            } else {
                err(KeyrHubError::IncorrectToken)
            }
        } else {
            err(KeyrHubError::MissingKeyrTokenHeader)
        }
    }
}
