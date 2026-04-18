//! A Rhai package that exposes a simple HTTP API to make requests.
//!
//! # Example
//!
//! ```rust
//! use rhai::Engine;
//! use rhai::packages::Package;
//! use rhai_http::HttpPackage;
//!
//! let mut engine = Engine::new();
//! HttpPackage::new().register_into_engine(&mut engine);
//! ```

use rhai::{def_package, plugin::*};

#[derive(Default, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Output {
    #[default]
    Text,
    Json,
}

#[derive(Clone, serde::Deserialize)]
struct Parameters {
    method: String,
    url: String,
    #[serde(default)]
    headers: rhai::Array,
    #[serde(default)]
    body: rhai::Dynamic,
    #[serde(default)]
    output: Output,
}

#[export_module]
pub mod api {
    use std::str::FromStr;

    /// A HTTP client that can execute HTTP requests. See `http::client` to create an instance.
    ///
    /// # rhai-autodocs:index:1
    pub type Client = reqwest::blocking::Client;

    /// Create a new HTTP client. Can be used to query HTTP endpoints.
    ///
    /// # Errors
    ///
    /// - TLS backend could not be initialized
    /// - Resolver could not load the system configuration
    ///
    /// # Example
    ///
    /// ```js
    /// let client = http::client();
    /// ```
    ///
    /// # rhai-autodocs:index:2
    #[rhai_fn(return_raw)]
    pub fn client() -> Result<Client, Box<rhai::EvalAltResult>> {
        reqwest::blocking::Client::builder()
            .build()
            .map_err(|error| error.to_string().into())
    }

    /// Execute an HTTP request.
    ///
    /// # Args
    ///
    /// - `parameter`: A map of parameters with the following fields:
    ///     - `method`: the method to use. (e.g. "POST", "GET", etc.)
    ///     - `url`: Endpoint to query.
    ///     - `headers`: Optional headers to add to the query, formatted as `"Name: Value"` strings.
    ///     - `body`: Optional body to add to the query.
    ///     - `output`: Output format: `"text"` (default) returns a String, `"json"` returns a Map.
    ///
    /// # Errors
    ///
    /// - The url failed to be parsed
    /// - Headers failed to be parsed
    /// - The request failed
    ///
    /// # Example
    ///
    /// ```js
    /// let client = http::client();
    ///
    /// let response = client.request(#{
    ///     method: "GET",
    ///     url: "http://example.com"
    /// });
    ///
    /// print(response)
    /// ```
    ///
    /// ```js
    /// let client = http::client();
    ///
    /// let response = client.request(#{
    ///     "method": "GET",
    ///     "url": "https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest?slug=bitcoin&convert=EUR",
    ///     "headers": [
    ///         "X-CMC_PRO_API_KEY: xxx",
    ///         "Accept: application/json"
    ///     ],
    ///     "output": "json",
    /// });
    ///
    /// print(response)
    /// ```
    ///
    /// # rhai-autodocs:index:3
    #[rhai_fn(global, pure, return_raw)]
    pub fn request(
        client: &mut Client,
        parameters: rhai::Map,
    ) -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        let Parameters {
            method,
            url,
            headers,
            body,
            output,
        } = rhai::serde::from_dynamic::<Parameters>(&parameters.into())?;

        let method = reqwest::Method::from_str(&method)
            .map_err::<Box<rhai::EvalAltResult>, _>(|error| error.to_string().into())?;

        client
            .request(method, url)
            .headers(
                headers
                    .iter()
                    .map(|header| {
                        if let Some((name, value)) = header.to_string().split_once(':') {
                            let name = name.trim();
                            let value = value.trim();

                            let name = reqwest::header::HeaderName::from_str(name).map_err::<Box<
                                EvalAltResult,
                            >, _>(
                                |error| error.to_string().into(),
                            )?;

                            let value = reqwest::header::HeaderValue::from_str(value)
                                .map_err::<Box<EvalAltResult>, _>(|error| {
                                    error.to_string().into()
                                })?;

                            Ok((name, value))
                        } else {
                            Err(format!("'{header}' is not a valid header").into())
                        }
                    })
                    .collect::<Result<reqwest::header::HeaderMap, Box<EvalAltResult>>>()?,
            )
            // FIXME: string or blob.
            .body(body.to_string())
            .send()
            .and_then(|response| match output {
                Output::Text => response.text().map(rhai::Dynamic::from),
                Output::Json => response.json::<rhai::Map>().map(rhai::Dynamic::from),
            })
            .map_err(|error| format!("{error:?}").into())
    }
}

def_package! {
    /// Package to build and send http requests.
    pub HttpPackage(_module) {} |> |engine| {
        // NOTE: since package modules items are registered in the global namespace,
        //       this is used to move the items in the `http` namespace.
        engine.register_static_module("http", rhai::exported_module!(api).into());
    }
}

#[cfg(test)]
pub mod test {
    use crate::HttpPackage;
    use rhai::packages::Package;

    fn setup_engine() -> rhai::Engine {
        let mut engine = rhai::Engine::new();

        HttpPackage::new().register_into_engine(&mut engine);
        engine
    }

    #[test]
    fn test_simple_query() {
        let engine = setup_engine();

        let body: String = engine
            .eval(
                r#"
let client = http::client();

client.request(#{ method: "GET", url: "http://example.com" })"#,
            )
            .unwrap();

        assert!(body
            .find("This domain is for use in documentation examples without needing permission.")
            .is_some());
    }

    #[test]
    fn test_simple_query_headers() {
        let engine = setup_engine();
        let body: rhai::Map = engine
            .eval(
                r#"
let client = http::client();

client.request(#{
    "method": "GET",
    "url": "https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest?slug=bitcoin&convert=EUR",
    "headers": [
        "X-CMC_PRO_API_KEY: xxx",
        "Accept: application/json"
    ],
    "output": "json",
})
"#,
            )
            .unwrap();

        println!("{body:#?}");
    }

    #[test]
    fn test_bad_header_name() {
        let engine = setup_engine();
        let error = engine
            .eval::<()>(
                r#"
let client = http::client();

client.request(#{
    "method": "GET",
    "url": "http://example.com",
    "headers": [
        "test/abc: xxx",
    ],
    "output": "json",
})
"#,
            )
            .err()
            .unwrap();

        assert!(matches!(
            *error,
            rhai::EvalAltResult::ErrorRuntime(dynamic, position) if dynamic.to_string() == "invalid HTTP header name" &&
            position == rhai::Position::new(4, 8)
        ));
    }

    #[test]
    fn test_bad_header_value() {
        let engine = setup_engine();
        // DEL (U+007F) is valid in Rhai strings but explicitly rejected by HeaderValue::from_str.
        let error = engine
            .eval::<()>(
                "let client = http::client();\n\
                 \n\
                 client.request(#{\n\
                     \"method\": \"GET\",\n\
                     \"url\": \"http://example.com\",\n\
                     \"headers\": [\n\
                         \"X-Custom: \x7F\",\n\
                     ],\n\
                 })",
            )
            .err()
            .unwrap();

        assert!(matches!(
            *error,
            rhai::EvalAltResult::ErrorRuntime(dynamic, position) if dynamic.to_string() == "failed to parse header value" &&
            position == rhai::Position::new(3, 8)
        ));
    }

    #[test]
    fn test_bad_header() {
        let engine = setup_engine();
        let error = engine
            .eval::<()>(
                r#"
let client = http::client();

client.request(#{
    "method": "GET",
    "url": "http://example.com",
    "headers": [
        "my header",
    ],
    "output": "json",
})
"#,
            )
            .err()
            .unwrap();

        assert!(matches!(
            *error,
            rhai::EvalAltResult::ErrorRuntime(dynamic, position) if dynamic.to_string() == "'my header' is not a valid header" &&
            position == rhai::Position::new(4, 8)
        ));
    }

    #[test]
    fn test_invalid_parameters() {
        let engine = setup_engine();
        let error = engine
            .eval::<()>(
                r#"
let client = http::client();

client.request(#{
    "output": "json",
})
"#,
            )
            .err()
            .unwrap();

        assert!(matches!(*error, rhai::EvalAltResult::ErrorParsing(_, _)));
    }

    #[test]
    fn test_invalid_method() {
        let engine = setup_engine();
        let error = engine
            .eval::<()>(
                r#"
let client = http::client();

client.request(#{
    "method": "INVALID METHOD",
    "url": "http://example.com",
})
"#,
            )
            .err()
            .unwrap();

        assert!(matches!(
            *error,
            rhai::EvalAltResult::ErrorRuntime(dynamic, position) if dynamic.to_string().contains("invalid HTTP method") &&
            position == rhai::Position::new(4, 8)
        ));
    }

    #[test]
    fn test_request_send_failure() {
        let engine = setup_engine();
        let error = engine
            .eval::<()>(
                r#"
let client = http::client();

client.request(#{
    "method": "GET",
    "url": "http://this-host-does-not-exist.invalid",
})
"#,
            )
            .err()
            .unwrap();

        assert!(matches!(
            *error,
            rhai::EvalAltResult::ErrorRuntime(_, position) if position == rhai::Position::new(4, 8)
        ));
    }
}
