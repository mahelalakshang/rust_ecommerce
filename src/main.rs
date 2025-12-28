use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod dtos;
mod error;
mod model;
mod utils;
mod web;

use config::Config;
use web::{auth, mw, post as post_handler, product as product_handler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env().await?;
    let state = Arc::new(config);

    // Run migrations on startup to ensure consistency
    sqlx::migrate!("./migrations")
        .run(&state.db_pool)
        .await
        .expect("Failed to run migrations");

    // Auth Routes
    let auth_routes = Router::new()
        .route("/signup", post(auth::signup_handler))
        .route("/login", post(auth::login_handler));

    // Post Routes (Protected)
    let post_routes = Router::new()
        .route(
            "/",
            post(post_handler::create_post).get(post_handler::get_posts),
        )
        .route("/{id}", get(post_handler::get_post_by_id))
        .route_layer(from_fn_with_state(state.clone(), mw::auth_guard));

    // Product Routes (Protected)
    let product_routes = Router::new()
        .route("/", post(product_handler::create_product))
        .route_layer(from_fn_with_state(state.clone(), mw::auth_guard));

    // Combine Routes
    let app = Router::new()
        .nest("/auth", auth_routes)
        .nest("/posts", post_routes)
        .nest("/products", product_routes)
        .with_state(state);

    // Start Server
    let listener = TcpListener::bind("127.0.0.1:3001").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
