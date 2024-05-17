use rhai::{def_package, plugin::*};

#[derive(Default, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Output {
    #[default]
    Text,
    Json,
}

#[derive(Clone, serde::Deserialize)]
pub struct GetParameters {
    url: String,
    #[serde(default)]
    headers: rhai::Map,
    #[serde(default)]
    body: rhai::Dynamic,
    #[serde(default)]
    output: Output,
}

#[export_module]
mod rhai_http {
    use std::str::FromStr;

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
    /// # rhai-autodocs:index:1
    #[rhai_fn(return_raw)]
    pub fn client() -> Result<Client, Box<rhai::EvalAltResult>> {
        reqwest::blocking::Client::builder()
            .build()
            .map_err(|error| error.to_string().into())
    }

    /// Execute a GET request.
    ///
    /// # Args
    ///
    /// - `parameter`: A map of parameters with the following fields:
    ///     - `url`: Endpoint to query.
    ///     - `headers`: Optional headers to add to the query.
    ///     - `body`: Optional body to add to the query.
    ///     - `output`: Output format of the response retrieved by the client, can either be 'text' or 'json'. Defaults to 'text'.
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
    /// let response = client.get(#{
    ///     url: "http://example.com"
    /// });
    ///
    /// print(response)
    /// ```
    ///
    /// # rhai-autodocs:index:2
    #[rhai_fn(global, pure, return_raw)]
    pub fn get(
        client: &mut Client,
        parameters: rhai::Map,
    ) -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        let GetParameters {
            url,
            headers,
            body,
            output,
        } = rhai::serde::from_dynamic::<GetParameters>(&parameters.into())?;

        client
            .get(url)
            .headers(
                headers
                    .iter()
                    .map(|(k, v)| {
                        let name = reqwest::header::HeaderName::from_str(k)
                            .map_err::<Box<EvalAltResult>, _>(|error| error.to_string().into())?;
                        let value = reqwest::header::HeaderValue::from_str(&v.to_string())
                            .map_err::<Box<EvalAltResult>, _>(|error| error.to_string().into())?;

                        Ok((name, value))
                    })
                    .collect::<Result<reqwest::header::HeaderMap, Box<EvalAltResult>>>()?,
            )
            .body(body.to_string())
            .send()
            .and_then(|response| match output {
                Output::Text => response.text().map(rhai::Dynamic::from),
                Output::Json => response.json::<rhai::Map>().map(rhai::Dynamic::from),
            })
            .map_err(|error| error.to_string().into())
    }
}

def_package! {
    /// Package to build and send http requests.
    pub HttpPackage(module) {
        combine_with_exported_module!(module, "http", rhai_http);
    }
}

#[cfg(test)]
pub mod test {
    use crate::HttpPackage;
    use rhai::packages::Package;

    #[test]
    fn simple_query() {
        let mut engine = rhai::Engine::new();

        HttpPackage::new().register_into_engine(&mut engine);

        let body: String = engine
            .eval(
                r#"
let client = client();

client.get(#{ url: "http://example.com" })"#,
            )
            .unwrap();

        assert_eq!(
            body,
            r#"<!doctype html>
<html>
<head>
    <title>Example Domain</title>

    <meta charset="utf-8" />
    <meta http-equiv="Content-type" content="text/html; charset=utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style type="text/css">
    body {
        background-color: #f0f0f2;
        margin: 0;
        padding: 0;
        font-family: -apple-system, system-ui, BlinkMacSystemFont, "Segoe UI", "Open Sans", "Helvetica Neue", Helvetica, Arial, sans-serif;
        
    }
    div {
        width: 600px;
        margin: 5em auto;
        padding: 2em;
        background-color: #fdfdff;
        border-radius: 0.5em;
        box-shadow: 2px 3px 7px 2px rgba(0,0,0,0.02);
    }
    a:link, a:visited {
        color: #38488f;
        text-decoration: none;
    }
    @media (max-width: 700px) {
        div {
            margin: 0 auto;
            width: auto;
        }
    }
    </style>    
</head>

<body>
<div>
    <h1>Example Domain</h1>
    <p>This domain is for use in illustrative examples in documents. You may use this
    domain in literature without prior coordination or asking for permission.</p>
    <p><a href="https://www.iana.org/domains/example">More information...</a></p>
</div>
</body>
</html>
"#
        );
    }
}
