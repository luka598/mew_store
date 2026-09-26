use axum::{body::Bytes, extract::State, http::{HeaderMap, HeaderValue, Method, StatusCode}, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};

use crate::store::SharedStore;

async fn cors(req: axum::extract::Request, next: Next) -> Response {
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

// router
pub async fn run(store: SharedStore) {
    let app = axum::Router::new()
        .route("/store/get", axum::routing::post(store_get))
        .route("/store/set", axum::routing::post(store_set))
        .route("/store/del", axum::routing::post(store_del))
        .route("/store/list", axum::routing::post(store_list))
        .layer(axum::middleware::from_fn(cors))
        .layer(axum::extract::DefaultBodyLimit::disable())
        .with_state(store);
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on http://{}/", addr);
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct Res<T: Serialize> {
    status: String,
    #[serde(flatten)]
    data: T,
}

// ReqStoreGet ResStoreGet store_get
#[derive(Deserialize)]
struct ReqStoreGet {
    u: String,
    k: String,
}

#[derive(Serialize)]
struct ResStoreGet {
    v: String,
}

async fn store_get(State(state): State<SharedStore>, body: Bytes) -> String {
    let Ok(req) = serde_json::from_slice::<ReqStoreGet>(&body) else {
        return serde_json::to_string(&Res {
            status: "invalid_request".to_string(),
            data: ResStoreGet { v: String::new() },
        })
        .unwrap();
    };
    let res = state.lock().await.get(&req.k);
    let (msg, v) = match res {
        Ok(v) => (String::new(), v),
        Err(_) => ("store_error".to_string(), String::new()),
    };
    serde_json::to_string(&Res {
        status: msg,
        data: ResStoreGet { v },
    })
    .unwrap()
}

// ReqStoreSet ResStoreSet store_set
#[derive(Deserialize)]
struct ReqStoreSet {
    u: String,
    k: String,
    v: String,
}

#[derive(Serialize)]
struct ResStoreSet {}

async fn store_set(State(state): State<SharedStore>, body: Bytes) -> String {
    let Ok(req) = serde_json::from_slice::<ReqStoreSet>(&body) else {
        return serde_json::to_string(&Res {
            status: "invalid_request".to_string(),
            data: ResStoreSet {},
        })
        .unwrap();
    };
    let res = state.lock().await.set(&req.k, &req.v);
    serde_json::to_string(&Res {
        status: if res.is_ok() { String::new() } else { "store_error".to_string() },
        data: ResStoreSet {},
    })
    .unwrap()
}

// ReqStoreList ResStoreList store_list
#[derive(Deserialize)]
struct ReqStoreList {
    u: String,
    q: String,
    limit: Option<i64>,
    cursor: Option<i64>,
}

#[derive(Serialize)]
struct ResStoreList {
    keys: Vec<String>,
}

async fn store_list(State(state): State<SharedStore>, body: Bytes) -> String {
    let Ok(req) = serde_json::from_slice::<ReqStoreList>(&body) else {
        return serde_json::to_string(&Res {
            status: "invalid_request".to_string(),
            data: ResStoreList { keys: Vec::new() },
        })
        .unwrap();
    };
    let limit = req.limit.unwrap_or(-1);
    let cursor = req.cursor.unwrap_or(0);
    let res = state.lock().await.list(&req.q, cursor, limit);
    let (msg, keys) = match res {
        Ok(keys) => (String::new(), keys),
        Err(_) => ("store_error".to_string(), Vec::new()),
    };
    serde_json::to_string(&Res {
        status: msg,
        data: ResStoreList { keys },
    })
    .unwrap()
}
#[derive(Deserialize)]
struct ReqStoreDel {
    u: String,
    k: String,
}

#[derive(Serialize)]
struct ResStoreDel {}

async fn store_del(State(state): State<SharedStore>, body: Bytes) -> String {
    let Ok(req) = serde_json::from_slice::<ReqStoreDel>(&body) else {
        return serde_json::to_string(&Res {
            status: "invalid_request".to_string(),
            data: ResStoreDel {},
        })
        .unwrap();
    };
    let res = state.lock().await.del(&req.k);
    serde_json::to_string(&Res {
        status: if res.is_ok() { String::new() } else { "store_error".to_string() },
        data: ResStoreDel {},
    })
    .unwrap()
}
