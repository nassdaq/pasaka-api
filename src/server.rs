//! The HTTP server: axum, Postgres-backed API keys, Swagger UI. This is the
//! native binary (Docker, `cargo run`); on wasm32-wasip1 the crate is a Barua
//! Function instead (see `function.rs`).

use axum::{Router, middleware};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{routes, utils};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let bible_data = routes::bible::BibleData::load();

    let shared_state = Arc::new((pool.clone(), bible_data));

    let auth_routes = routes::auth::routes().with_state(pool.clone());

    let bible_routes = routes::bible::routes();

    let secured_bible_routes = bible_routes
        .layer(middleware::from_fn_with_state(shared_state.clone(), utils::auth::require_api_key));

    let swagger_ui = SwaggerUi::new("/swagger-ui")
      .url("/api-docs/openapi.json", routes::ApiDoc::openapi());

    let app = Router::new()
      .merge(swagger_ui)
      .merge(secured_bible_routes)
      .merge(auth_routes)
      .layer(cors)
      .layer(CompressionLayer::new())
      .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
