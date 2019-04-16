// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use jsonrpc_core::error::Error as CoreError;
use reqwest::Error as ReqwestError;

#[derive(Debug)]
pub enum Error {
    OptionNone,
    Custom(String),
    Core(CoreError),
    Serde(String),
    Reqwest(ReqwestError),
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ::std::error::Error for Error {}

pub type Result<T> = ::std::result::Result<T, Error>;

impl Error {
    pub fn none() -> Self {
        Error::OptionNone
    }

    pub fn custom(msg: &str) -> Self {
        Error::Custom(msg.to_owned())
    }

    pub fn serde(msg: &str) -> Self {
        Error::Serde(msg.to_owned())
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Error::Core(err)
    }
}

impl From<ReqwestError> for Error {
    fn from(err: ReqwestError) -> Self {
        Error::Reqwest(err)
    }
}
