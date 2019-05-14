// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use reqwest::{header::HeaderMap, IntoUrl};
use reqwest::{
    Client as RawClient, ClientBuilder as RawClientBuilder, RequestBuilder as RawRequestBuilder,
    Response as RawResponse,
};

use jsonrpc_sdk_prelude::{jsonrpc_core::Response, CommonPart, JsonRpcRequest, Result};

pub struct Client(RawClient);
pub struct ClientBuilder(RawClientBuilder);
pub struct RequestBuilder(RawRequestBuilder);

impl Client {
    pub fn new() -> Self {
        Client(RawClient::new())
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder(RawClient::builder())
    }

    pub fn post<U>(&self, url: U) -> RequestBuilder
    where
        U: IntoUrl,
    {
        RequestBuilder(self.0.post(url))
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    pub fn build(self) -> Result<Client> {
        Ok(Client(self.0.build()?))
    }

    pub fn tcp_nodelay(self) -> Self {
        ClientBuilder(self.0.tcp_nodelay())
    }

    pub fn default_headers(self, headers: HeaderMap) -> Self {
        ClientBuilder(self.0.default_headers(headers))
    }

    pub fn gzip(self, enable: bool) -> Self {
        ClientBuilder(self.0.gzip(enable))
    }

    pub fn connect_timeout(self, timeout: ::std::time::Duration) -> Self {
        ClientBuilder(self.0.connect_timeout(timeout))
    }
}

impl RequestBuilder {
    pub fn send<T>(self, content: T, common: CommonPart) -> Result<T::Output>
    where
        T: JsonRpcRequest,
    {
        normal_error!(content.to_single_request(common)).and_then(|request| {
            self.0
                .json(&request)
                .send()
                .and_then(RawResponse::error_for_status)
                .and_then(|mut r| r.json::<Response>())
                .map_err(std::convert::Into::into)
                .and_then(T::parse_single_response)
        })
    }
}
