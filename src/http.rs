//! This module re-exports some types used for http request mocking and stubbing.
//! One http handler is used globally for all http requests, whether it is stubbed or mocked.
//!
//! You must specify a http handler through [`mock_http_handler`](super::ComponentCompositionBuilder::mock_http_handler) or [`stub_http_handler`](super::ComponentCompositionBuilder::stub_http_handler),
//! otherwise you will get a panic when a component tries to send a http request.
//!
//! You can mock a http handler like so:
//! ```no_run
//! # mod bindings {
//! #     wasmtime_testing_helper::wasmtime::component::bindgen!({
//! #         inline: r"
//! #             package namespace:%package;
//! #
//! #             interface %interface {
//! #                 function: func(length: u32) -> string;
//! #             }
//! #
//! #             world main {
//! #                 export %interface;
//! #             }
//! #         "
//! #     });
//! #
//! #     wasmtime_testing_helper::setup!(Main);
//! # }
//! let mut harness = bindings::harness();
//! harness.mock_http_handler(
//!     Box::new(|request, config| {
//!        Box::pin(async move {
//!             Ok(hyper::Response::new(request.into_body().await.unwrap()))
//!         })
//!    })
//! );
//! ```
//!
//! Or, if you have a static response, you can stub the http handler like so:
//! ```no_run
//! # mod bindings {
//! #     wasmtime_testing_helper::wasmtime::component::bindgen!({
//! #         inline: r"
//! #             package namespace:%package;
//! #
//! #             interface %interface {
//! #                 function: func(length: u32) -> string;
//! #             }
//! #
//! #             world main {
//! #                 export %interface;
//! #             }
//! #         "
//! #     });
//! #
//! #     wasmtime_testing_helper::setup!(Main);
//! # }
//! let mut harness = bindings::harness();
//! harness.stub_http_handler(
//!     Ok(hyper::Response::new(String::from("All good!")))
//! );
//! ```
//!
//! Currently, sending real http requests is not implemented. In the case where that is required,
//! you can mock the http handler with a real http request implementation.

pub extern crate hyper;
use std::{pin::Pin, time::Duration};

use http_body_util::BodyExt;
use hyper::{Request, Response};
use wasmtime_wasi_http::p2::{WasiHttpHooks, types::IncomingResponse};
pub use wasmtime_wasi_http::p2::{bindings::http::types::ErrorCode, types::OutgoingRequestConfig};

pub type HttpRequest =
    hyper::Request<Pin<Box<dyn Future<Output = Result<hyper::body::Bytes, ErrorCode>> + Send>>>;
pub type HttpResponse =
    Pin<Box<dyn Send + Future<Output = Result<hyper::Response<hyper::body::Bytes>, ErrorCode>>>>;
pub type HttpHandler =
    Box<dyn Send + Sync + FnMut(HttpRequest, OutgoingRequestConfig) -> HttpResponse>;

#[derive(Default)]
pub(crate) struct HttpHooks {
    request_handler: Option<HttpHandler>,
    between_bytes_timeout: Duration,
}

impl HttpHooks {
    pub(crate) fn set_request_handler(&mut self, request_handler: HttpHandler) {
        self.request_handler = Some(Box::new(request_handler))
    }

    pub(crate) fn new() -> Self {
        Self {
            request_handler: None,
            between_bytes_timeout: Duration::from_secs(5),
        }
    }
}

impl WasiHttpHooks for HttpHooks {
    fn send_request(
        &mut self,
        request: hyper::Request<wasmtime_wasi_http::p2::body::HyperOutgoingBody>,
        config: wasmtime_wasi_http::p2::types::OutgoingRequestConfig,
    ) -> wasmtime_wasi_http::p2::HttpResult<wasmtime_wasi_http::p2::types::HostFutureIncomingResponse>
    {
        let (parts, body) = request.into_parts();
        let future = self
            .request_handler
            .as_mut()
            .expect("no http request handler was set")(
            Request::from_parts(
                parts,
                Box::pin(async { body.collect().await.map(|v| v.to_bytes()) }),
            ),
            config,
        );
        let between_bytes_timeout = self.between_bytes_timeout;
        Ok(
            wasmtime_wasi_http::p2::types::HostFutureIncomingResponse::pending(
                wasmtime_wasi::runtime::spawn(async move {
                    Ok(future.await.map(|req| {
                        let (parts, body) = req.into_parts();
                        let boxed_body = BodyExt::boxed_unsync(
                            http_body_util::Full::new(body)
                                .map_err(|err| ErrorCode::InternalError(Some(err.to_string()))),
                        );
                        IncomingResponse {
                            resp: Response::from_parts(parts, boxed_body),
                            worker: None,
                            between_bytes_timeout,
                        }
                    }))
                }),
            ),
        )
    }
}
