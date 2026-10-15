use axum::{
  extract::{Path,State},
  routing::{get,post},
  Json,Router, 
};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

use crate::models::bible::{
  BibleResponse,
  Book,
  Chapter,
  SearchQuery,
  SearchResult,
  Verse,
  ErrorResponse
};
use axum::extract::Query;
use std::collections::HashMap;
use serde_json::json;



/// The books, preloaded once from the index build.rs compiles in; nothing is
/// read from disk at run time.
pub struct BibleData {
  books: Vec<Book>,
}

impl BibleData {
  pub fn load() -> Self {
    Self { books: crate::scripture::books().map(crate::scripture::BookRef::to_book).collect() }
  }

  fn get_books(&self) -> &Vec<Book>{
    &self.books
  }
  fn get_book(&self,book_id: &str) -> Option<&Book>{
    self.books.iter().find(|b| b.id == book_id)
  }
}

type AppState = Arc<(PgPool, BibleData)>;

#[utoipa::path(
  get,
  path="/books",
  responses(
    (status=200,description = "List of all book",body = BibleResponse<Vec<Book>>),
    (status =401,description = "Unauthorized",body = ErrorResponse)

  ),
  security(
    ("api_key" =[])
  )
)]

async fn get_books(
  State(state): State<AppState>,
  Query(params): Query<HashMap<String, String>>,
) -> Json<BibleResponse<Value>> {
  let bible_data = &state.1;
  let full = params.get("full").map(|v| v == "true").unwrap_or(false);

  if full {
      Json(BibleResponse {
          success: true,
          data: serde_json::to_value(bible_data.get_books()).unwrap(),
      })
  } else {
     
      let simple_books: Vec<_> = bible_data.get_books().iter()
          .map(|b| {
              json!({
                  "id": b.id,
                  "name": b.name,
              })
          }).collect();

      Json(BibleResponse {
          success: true,
          data: json!(simple_books),
      })
  }
}


#[utoipa::path(
  get,
  path = "/books/{book_id}",
  params(
      ("book_id" = String, Path, description = "Book ID")
  ),
  responses(
      (status = 200, description = "Book details", body = BibleResponse<Book>),
      (status = 404, description = "Book not found", body = ErrorResponse),
      (status = 401, description = "Unauthorized", body = ErrorResponse)
  ),
  security(
      ("api_key" = [])
  )
)]
async fn get_book(
  State(state):State<AppState>,
  Path(book_id):Path<String>,

) -> Json<Value> {
  let bible_data = &state.1;

  match bible_data.get_book(&book_id){
    Some(book) => Json(serde_json::json!({
      "success":true,
      "data":book,
    })),
    None => Json(serde_json::json!({
      "success":false,
      "error":format!("Book '{}' not found",book_id)
    })),
  }
}

#[utoipa::path(
  get,
  path="/book/{book_id}/{chapter}",
  params(
    
    ("book_id"=String, Path, description="Book ID"),
    ("chapter"=String, Path, description="Chapter number")

  ),
  responses(
    (status = 200, description="Chapter details",body =BibleResponse<Chapter>),
    (status = 404,description="Book or Chapter not found",body=ErrorResponse),
    (status = 401,description="Unauthorized",body=ErrorResponse),
  ),
  security(
    ("api_key"=[])
  )
)]
async fn get_chapter(
  State(state): State<AppState>,
  Path((book_id,chapter)): Path<(String,String)>,
) -> Json<Value>{
  let bible_data = &state.1;
  
  if let  Some(book) = bible_data.get_book(&book_id){
    if let Some(chap) = book.chapters.iter().find(|c| c.chapter == chapter){
      return Json(serde_json::json!({
        "success":true,
        "data":chap,
      }));
    }
    return Json(serde_json::json!({
      "success":false,
      "error":format!("Chapter {} not found in book '{}",chapter,book_id)
    }));

  }
  Json(serde_json::json!({
    "success":false,
    "error":format!("Book '{}' not found",book_id)
  }))
}

#[utoipa::path(
  get,
  path = "/books/{book_id}/{chapter}/{verse}",
  params(
      ("book_id" = String, Path, description = "Book ID"),
      ("chapter" = String, Path, description = "Chapter number"),
      ("verse" = String, Path, description = "Verse number")
  ),
  responses(
      (status = 200, description = "Verse details", body = BibleResponse<Verse>),
      (status = 404, description = "Book, chapter, or verse not found", body = ErrorResponse),
      (status = 401, description = "Unauthorized", body = ErrorResponse)
  ),
  security(
      ("api_key" = [])
  )
)]
async fn get_verse(
  State(state): State<AppState>,
  Path((book_id, chapter, verse)): Path<(String, String, String)>,
) -> Json<Value> {
  let bible_data = &state.1;
  
  if let Some(book) = bible_data.get_book(&book_id) {
      if let Some(chap) = book.chapters.iter().find(|c| c.chapter == chapter) {
          if let Some(v) = chap.verses.iter().find(|v| v.verse == verse) {
              return Json(serde_json::json!({
                  "success": true,
                  "data": v
              }));
          }
          
          return Json(serde_json::json!({
              "success": false,
              "error": format!("Verse {} not found in chapter {} of book '{}'", verse, chapter, book_id)
          }));
      }
      
      return Json(serde_json::json!({
          "success": false,
          "error": format!("Chapter {} not found in book '{}'", chapter, book_id)
      }));
  }
  
  Json(serde_json::json!({
      "success": false,
      "error": format!("Book '{}' not found", book_id)
  }))
}

#[utoipa::path(
  post,
  path = "/search",
  request_body = SearchQuery,
  responses(
      (status = 200, description = "Search results", body = BibleResponse<Vec<SearchResult>>),
      (status = 401, description = "Unauthorized", body = ErrorResponse)
  ),
  security(
      ("api_key" = [])
  )
)]
async fn search_bible(
  State(_state): State<AppState>,
  Json(payload): Json<SearchQuery>,
) -> Json<BibleResponse<Vec<SearchResult>>> {
  let results = crate::scripture::search(&payload.query);
  
  Json(BibleResponse {
    success: true,
    data: results,
  })
}

pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/books", get(get_books))
    .route("/books/{book_id}", get(get_book))          
    .route("/books/{book_id}/{chapter}", get(get_chapter)) 
    .route("/books/{book_id}/{chapter}/{verse}", get(get_verse))  
    .route("/search", post(search_bible))
}