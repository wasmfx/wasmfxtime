use anyhow::Context;
use wasi_http_tests::bindings::wasi::http::types::{Method, Scheme};

fn main() {
    let addr = std::env::var("HTTP_SERVER").unwrap();
    let res = wasi_http_tests::request(Method::Put, Scheme::Http, &addr, "/put", Some(&[]), None)
        .context("/put")
        .unwrap();

    println!("/put: {res:?}");
    assert_eq!(res.status, 200);
    let method = res.header("x-wasmtime-test-method").unwrap();
    assert_eq!(std::str::from_utf8(method).unwrap(), "PUT");
    assert_eq!(res.body, b"");
}
