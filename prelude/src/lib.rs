// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use jsonrpc_core;
pub use serde_json;

pub use jsonrpc_sdk_macros::jsonrpc_interfaces;

mod error;
mod kernel;

pub use error::{Error, Result};
pub use kernel::{CommonPart, JsonRpcRequest};
