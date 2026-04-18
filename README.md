# rhai-http

[![Crates.io](https://img.shields.io/crates/v/rhai-http)](https://crates.io/crates/rhai-http)
[![License](https://img.shields.io/github/license/ltabis/rhai-http)](./LICENSE)
[![CI](https://github.com/ltabis/rhai-http/actions/workflows/ci.yaml/badge.svg)](https://github.com/ltabis/rhai-http/actions/workflows/ci.yaml)
[![docs.rs](https://docs.rs/rhai-http/badge.svg)](https://docs.rs/rhai-http)

A Rhai package that exposes a simple http API to make requests.
Uses [rhai-autodocs](https://github.com/ltabis/rhai-autodocs) to build documentation for the Rhai API.

### Simple GET request

```js
let client = http::client();

client.request(#{ method: "GET", url: "http://example.com" })
```

### GET request with headers and JSON output

```js
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
```
