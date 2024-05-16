use rhai::{def_package, plugin::*};

#[derive(Clone, serde::Deserialize)]
pub struct GetParameters {
    url: String,
    #[serde(default)]
    headers: rhai::Map,
    #[serde(default)]
    body: String,
}

#[export_module]
mod rhai_http {
    pub type Client = reqwest::blocking::Client;

    /// Create a new HTTP client.
    ///
    /// # rhai-autodocs:index:1
    #[rhai_fn(return_raw)]
    pub fn client() -> Result<Client, Box<rhai::EvalAltResult>> {
        reqwest::blocking::Client::builder()
            .build()
            .map_err(|error| error.to_string().into())
    }

    // TODO: when does json should be returned ?
    /// Execute a GET request.
    ///
    /// # rhai-autodocs:index:2
    #[rhai_fn(global, pure, return_raw)]
    pub fn get(
        client: &mut Client,
        parameters: rhai::Map,
    ) -> Result<String, Box<rhai::EvalAltResult>> {
        let GetParameters { url, headers, body } =
            rhai::serde::from_dynamic::<GetParameters>(&parameters.into())?;

        client
            .get(url)
            .headers(
                headers
                    .iter()
                    .map(|(k, v)| {
                        let name = k.to_string().try_into().unwrap();
                        let value = v.to_string().try_into().unwrap();

                        Ok((name, value))
                    })
                    .collect::<Result<reqwest::header::HeaderMap, reqwest::Error>>()
                    .map_err::<Box<rhai::EvalAltResult>, _>(|error| error.to_string().into())?,
            )
            .body(body)
            .send()
            .and_then(|response| response.text())
            .map_err(|error| error.to_string().into())
    }
}

def_package! {
    /// Package to build and send http requests.
    pub HttpPackage(module) {
        combine_with_exported_module!(module, "http", rhai_http);
    }
}

/// Export the module for the dynamic library.
#[no_mangle]
pub extern "C" fn module_entrypoint() -> rhai::Shared<rhai::Module> {
    // The seed must be the same as the one used in the program that will
    // load this module.
    rhai::config::hashing::set_hashing_seed(Some([1, 2, 3, 4])).unwrap();

    rhai::exported_module!(rhai_http).into()
}

#[cfg(test)]
pub mod test {
    use rhai::packages::Package;

    use crate::HttpPackage;

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
