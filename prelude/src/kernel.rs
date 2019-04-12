// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::convert::{TryFrom, TryInto};

use jsonrpc_core::{
    id::Id,
    params::Params,
    request::{Call, MethodCall, Notification, Request},
    response::{Failure, Output, Response, Success},
    types::to_string,
    version::Version,
    Value,
};
use log::trace;

use crate::{Error, Result};

pub struct CommonPart {
    pub jsonrpc: Option<Version>,
    pub id_opt: Option<Id>,
}

impl Default for CommonPart {
    fn default() -> Self {
        Self::num(0)
    }
}

impl CommonPart {
    pub fn num(n: u64) -> Self {
        Self {
            jsonrpc: Some(Version::V2),
            id_opt: Some(Id::Num(n)),
        }
    }

    pub fn str(s: String) -> Self {
        Self {
            jsonrpc: Some(Version::V2),
            id_opt: Some(Id::Str(s)),
        }
    }
}

pub trait JsonRpcRequest
where
    Self: TryInto<Params> + Sized,
{
    type Output: TryFrom<Value>;

    fn method() -> &'static str;

    fn to_string(self, c: CommonPart) -> Result<String> {
        self.to_single_request(c).and_then(|sc| {
            to_string(&sc).map_err(|_| Error::serde("failed to convert a single request to string"))
        })
    }

    fn to_single_request(self, c: CommonPart) -> Result<Request> {
        self.to_call(c).map(Request::Single)
    }

    fn to_call(self, c: CommonPart) -> Result<Call> {
        let CommonPart { jsonrpc, id_opt } = c;
        let method = Self::method().to_owned();
        self.try_into()
            .map_err(|_| Error::serde("failed to parse a request core"))
            .map(|params: Params| {
                if let Some(id) = id_opt {
                    Call::MethodCall(MethodCall {
                        jsonrpc,
                        method,
                        params,
                        id,
                    })
                } else {
                    Call::Notification(Notification {
                        jsonrpc,
                        method,
                        params,
                    })
                }
            })
    }

    fn parse_single_response(response: Response) -> Result<Self::Output> {
        match response {
            Response::Single(output) => match output {
                Output::Success(success) => {
                    let Success {
                        jsonrpc,
                        result,
                        id,
                    } = success;
                    trace!("Success {{ jsonrpc: {:#?}, id: {:#?} }}", jsonrpc, id);
                    result
                        .try_into()
                        .map_err(|_| Error::custom("failed to parse the result: {}"))
                }
                Output::Failure(failure) => {
                    let Failure { jsonrpc, error, id } = failure;
                    trace!("Failure {{ jsonrpc: {:#?}, id: {:#?} }}", jsonrpc, id);
                    Err(error.into())
                }
            },
            Response::Batch(_) => Err(Error::custom("could not be a batch response")),
        }
    }
}
