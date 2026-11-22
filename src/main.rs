use axum::Router;
use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use reel_vault::video::handle_upload;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST]);

    let app = Router::new()
        .route("/health", get(|| async { "Server is running" }))
        .route("/upload", post(handle_upload))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
