// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use futures::Future;
pub use reqwest::r#async::{
    Client as RawClient, ClientBuilder as RawClientBuilder, Decoder, Request as RawRequest,
    RequestBuilder as RawRequestBuilder, Response as RawResponse,
};
use reqwest::IntoUrl;
pub use reqwest::{Error as RawError, Method as RawMethod};

use jsonrpc_sdk_prelude::{jsonrpc_core::Response, CommonPart, Error, JsonRpcRequest};

pub struct Client {
    inner: RawClient,
}

impl Client {
    pub fn new() -> Self {
        Self {
            inner: RawClient::new(),
        }
    }

    pub fn request<U>(&self, method: RawMethod, url: U) -> RequestBuilder
    where
        U: IntoUrl,
    {
        RequestBuilder::new(self.inner.request(method, url))
    }

    pub fn post<U>(&self, url: U) -> RequestBuilder
    where
        U: IntoUrl,
    {
        self.request(RawMethod::POST, url)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RequestBuilder {
    inner: RawRequestBuilder,
}

impl RequestBuilder {
    pub fn new(inner: RawRequestBuilder) -> Self {
        Self { inner }
    }

    pub fn send<T>(
        self,
        content: T,
        common: CommonPart,
    ) -> impl Future<Item = T::Output, Error = Error>
    where
        T: JsonRpcRequest,
    {
        let request = content.to_single_request(common).unwrap();
        self.inner
            .json(&request)
            .send()
            .and_then(RawResponse::error_for_status)
            .and_then(|mut r| r.json::<Response>())
            .map_err(std::convert::Into::into)
            .and_then(T::parse_single_response)
    }
}
