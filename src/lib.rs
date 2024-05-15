use rhai::plugin::*;

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

    /// Execute a GET request.
    ///
    /// # rhai-autodocs:index:2
    #[rhai_fn(pure, return_raw)]
    pub fn get(
        client: &mut Client,
        parameters: rhai::Map,
    ) -> Result<rhai::Map, Box<rhai::EvalAltResult>> {
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
            .map(|response| response.json().unwrap())
            .map_err(|error| error.to_string().into())
    }
}

/// Export the rhai_http module.
#[no_mangle]
pub extern "C" fn module_entrypoint() -> rhai::Shared<rhai::Module> {
    // The seed must be the same as the one used in the program that will
    // load this module.
    rhai::config::hashing::set_hashing_seed(Some([1, 2, 3, 4])).unwrap();

    rhai::exported_module!(rhai_http).into()
}
