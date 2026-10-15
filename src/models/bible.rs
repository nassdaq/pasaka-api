use serde::{Deserialize,Serialize};

// `ToSchema` (Swagger) exists only where the HTTP server does; on wasm32-wasip1
// these are plain serde types.

/// The shape of src/data/swahili_bible.json. Only the OpenAPI document still
/// refers to it: the data itself is indexed by build.rs.
#[cfg(not(target_os = "wasi"))]
#[derive(Debug,Serialize,Deserialize,utoipa::ToSchema)]
pub struct BibleRoot{
  #[serde(rename ="BIBLEBOOK")]
  pub books:Vec<Book>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[cfg_attr(not(target_os = "wasi"), derive(utoipa::ToSchema))]
pub struct Book {
  #[serde(rename = "book_number")]
  pub id: String,
  #[serde(rename = "book_name")]
  pub name: String,
  #[serde(rename = "CHAPTER")]
  pub chapters: Vec<Chapter>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[cfg_attr(not(target_os = "wasi"), derive(utoipa::ToSchema))]
pub struct Chapter {
  #[serde(rename="chapter_number")]
  pub chapter: String,
  #[serde(rename="VERSES")]
  pub verses: Vec<Verse>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[cfg_attr(not(target_os = "wasi"), derive(utoipa::ToSchema))]
pub struct Verse {
    #[serde(rename = "verse_number")]
    pub verse: String,
    #[serde(rename = "verse_text")]
    pub text: String,
}

#[derive(Debug,Serialize,Deserialize)]
#[cfg_attr(not(target_os = "wasi"), derive(utoipa::ToSchema))]
pub struct SearchResult{
  pub book:String,
  pub chapter:String,
  pub verse: String,
  pub text: String,
}

#[derive(Debug,Serialize)]
#[cfg_attr(not(target_os = "wasi"), derive(utoipa::ToSchema))]
pub struct BibleResponse<T>{
  pub success: bool,
  pub data: T,
}

#[derive(Debug,Serialize)]
#[cfg_attr(not(target_os = "wasi"), derive(utoipa::ToSchema))]
pub struct ErrorResponse {
  pub success:bool,
  pub error: String,
}

#[cfg(not(target_os = "wasi"))]
#[derive(Debug,Deserialize,utoipa::ToSchema)]
pub struct SearchQuery {
  pub query: String,
}
