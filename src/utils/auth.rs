use axum::{
  body::Body,
  extract::State,
  http::{Request, StatusCode},
  middleware::Next,
  response::Response,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::models::bible::ErrorResponse;


pub async fn require_api_key<S>(
  State(state): State<Arc<(PgPool, S)>>,
  request: Request<Body>,
  next: Next,
) -> Result<Response, StatusCode>
where
  S: Send + Sync + 'static,
{

  let api_key = request
      .headers()
      .get("x-api-key")
      .and_then(|header| header.to_str().ok())
      .map(|s| s.to_string());

 
  let api_key = match api_key {
      Some(key) => key,
      None => {
          let error = ErrorResponse {
              success: false,
              error: "API key is required".to_string(),
          };
          
          return Ok(Response::builder()
              .status(StatusCode::UNAUTHORIZED)
              .header("Content-Type", "application/json")
              .body(Body::from(
                  serde_json::to_string(&error).unwrap(),
              ))
              .unwrap());
      }
  };

 
  let pool = &state.0;
  
  let result = sqlx::query("SELECT id FROM users WHERE api_key = $1")
      .bind(&api_key)
      .fetch_optional(pool)
      .await;

  match result {
      Ok(Some(_)) => {
          
        Ok(next.run(request).await)
      }
      Ok(None) => {
       
          let error = ErrorResponse {
              success: false,
              error: "Invalid API key".to_string(),
          };
          
          Ok(Response::builder()
              .status(StatusCode::UNAUTHORIZED)
              .header("Content-Type", "application/json")
              .body(Body::from(
                  serde_json::to_string(&error).unwrap(),
              ))
              .unwrap())
      }
      Err(_) => {
       
          let error = ErrorResponse {
              success: false,
              error: "Internal server error".to_string(),
          };
          
          Ok(Response::builder()
              .status(StatusCode::INTERNAL_SERVER_ERROR)
              .header("Content-Type", "application/json")
              .body(Body::from(
                  serde_json::to_string(&error).unwrap(),
              ))
              .unwrap())
      }
  }
}