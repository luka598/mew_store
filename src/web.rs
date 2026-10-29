use axum::http::Response;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::{body::Bytes, extract::State};
use std::collections::HashMap;

use crate::store::{Operation, Request, Store};

pub type SharedStore = std::sync::Arc<Store>;

async fn cors(req: axum::extract::Request, next: Next) -> Response<axum::body::Body> {
    let mut headers = HeaderMap::new();
    headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
    headers.insert("access-control-allow-methods", HeaderValue::from_static("*"));
    headers.insert("access-control-allow-headers", HeaderValue::from_static("*"));

    if req.method() == Method::OPTIONS {
        let mut res = Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(axum::body::Body::empty())
            .unwrap();
        res.headers_mut().extend(headers);
        return res;
    }

    let mut res = next.run(req).await;
    res.headers_mut().extend(headers);
    res
}

pub async fn run(store: SharedStore) {
    let app = axum::Router::new()
        .route("/store", axum::routing::post(store_op))
        .layer(axum::middleware::from_fn(cors))
        .layer(axum::extract::DefaultBodyLimit::disable())
        .with_state(store);
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on http://{}/", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn store_op(State(state): State<SharedStore>, body: Bytes) -> String {
    let Ok(op) = serde_json::from_slice::<Operation>(&body) else {
        return serde_json::to_string(&crate::store::Response::Error("invalid_request".to_string())).unwrap();
    };

    let res = state.run(Request::Exec { op, extra: HashMap::new() });
    match res {
        crate::store::Response::None => String::new(),
        _ => serde_json::to_string(&res).unwrap(),
    }
}