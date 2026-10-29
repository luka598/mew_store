mod store;
mod web;
mod feat;

#[tokio::main]
async fn main() {
    let s = std::sync::Arc::new(
        store::Store::new().add_middleware(feat::sqlite::SqliteStoreMiddleware),
    );
    web::run(s).await;
}