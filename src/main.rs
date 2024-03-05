use std::{env, fs};
use std::future::Future;
use std::pin::Pin;

use http_body_util::Full;
use hyper_util::rt::TokioIo;
use hyper::{Request, Response};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::Service;
use tokio::net::TcpListener;

mod http_codes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listen_addr = env::var("LISTEN_ADDR").ok().unwrap_or_else(|| String::from("0.0.0.0"));
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or_else(|| 7878);

    println!("Trying to bind to {listen_addr}:{port}...");
    let listener = TcpListener::bind(format!("{listen_addr}:{port}")).await?;
    println!("Listening on {listen_addr}:{port}");

    let errorpage_template: &'static str = Box::leak(fs::read_to_string("error.html").unwrap().into_boxed_str());

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            http1::Builder::new()
                .serve_connection(io, Svc { errorpage_template })
                .await
        });
    }
}

struct Svc {
    errorpage_template: &'static str,
}

impl Service<Request<Incoming>> for Svc {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: Request<Incoming>) -> Self::Future {
        let mut status_code = request.uri().path().replace("/", "").parse::<u16>().unwrap_or(404);
        if status_code < 100 || status_code > 999 {
            status_code = 404;
        }

        let response = Ok(Response::builder()
            .status(status_code)
            .body(Full::new(Bytes::from(self.errorpage_template
                .replace("{error}", status_code.to_string().as_str())
                .replace("{message}", http_codes::get_code(status_code).unwrap_or("Unknown error occurred"))
                .replace("{debug}", request.headers()
                    .iter()
                    .map(|(k, v)| format!("{}: {}\r\n", k, v.to_str().unwrap()))
                    .collect::<String>()
                    .as_str()))
            ))
            .unwrap()
        );
        Box::pin(async { response })
    }
}
