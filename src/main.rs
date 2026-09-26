mod store;
mod web;

#[tokio::main]
async fn main() {
    let s = std::sync::Arc::new(tokio::sync::Mutex::new(store::Store::new().unwrap()));
    web::run(s).await;
}
