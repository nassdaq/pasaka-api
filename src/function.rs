//! The crate as a Barua Function: one request on stdin, one JSON answer on stdout.
//!
//! Barua Functions (fn.barua.app) compiles the repository to `wasm32-wasip1`
//! and runs a fresh instance per call with the request body on stdin; whatever
//! is printed is the response. There is no network and no filesystem, so the
//! HTTP server, Postgres and API keys do not exist here. The Bible is compiled
//! in (`build.rs`, `scripture.rs`) and every route becomes a body:
//!
//! ```text
//! (empty)                                   the books, id and name
//! books/1 · books/1/1 · books/66/1/1        a book, a chapter, a verse
//! {"book":"66","chapter":"1","verse":"1"}   the same, as JSON
//! {"query":"upendo"} · any other text       search
//! {"full":true}                             every book with every verse
//! ```
//!
//! Answers have the HTTP API's shape: `{"success":true,"data":…}` or
//! `{"success":false,"error":"…"}`. The exit code is always 0 so the platform
//! returns the JSON as it is; a non-zero exit would replace it with its own.

// Natively this module is only reached by its tests.
#![cfg_attr(not(target_os = "wasi"), allow(dead_code))]

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::models::bible::{BibleResponse, ErrorResponse};
use crate::scripture;

/// Read stdin, answer on stdout.
pub fn run() {
    use std::io::{Read, Write};

    let mut input = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut input);
    let output = handle(&String::from_utf8_lossy(&input));

    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(output.as_bytes());
    let _ = stdout.flush();
}

/// The JSON answer for one request body.
pub fn handle(body: &str) -> String {
    match parse(body).and_then(answer) {
        Ok(json) => json,
        Err(error) => serde_json::to_string(&ErrorResponse { success: false, error })
            .expect("an error response serialises"),
    }
}

enum Request {
    Books { full: bool },
    Book(String),
    Chapter(String, String),
    Verse(String, String, String),
    Search(String),
}

/// What a JSON body may say. Unknown keys are refused so a typo such as
/// `{"books":"1"}` is an error instead of the book list.
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct JsonBody {
    book: Option<Value>,
    chapter: Option<Value>,
    verse: Option<Value>,
    query: Option<String>,
    full: bool,
}

fn parse(body: &str) -> Result<Request, String> {
    let body = body.trim();
    if body.is_empty() {
        return Ok(Request::Books { full: false });
    }
    if body.starts_with('{') {
        return parse_json(body);
    }
    Ok(parse_path(body).unwrap_or_else(|| Request::Search(body.to_string())))
}

fn parse_json(body: &str) -> Result<Request, String> {
    let json: JsonBody = serde_json::from_str(body).map_err(|e| {
        format!("the body must be JSON like {{\"book\":\"1\",\"chapter\":\"1\",\"verse\":\"1\"}} or {{\"query\":\"upendo\"}}: {e}")
    })?;
    if let Some(query) = json.query {
        return Ok(Request::Search(query));
    }
    let book = number("book", json.book)?;
    let chapter = number("chapter", json.chapter)?;
    let verse = number("verse", json.verse)?;
    match (book, chapter, verse) {
        (Some(b), Some(c), Some(v)) => Ok(Request::Verse(b, c, v)),
        (Some(b), Some(c), None) => Ok(Request::Chapter(b, c)),
        (Some(b), None, None) => Ok(Request::Book(b)),
        (None, None, None) => Ok(Request::Books { full: json.full }),
        _ => Err("a chapter needs its book, and a verse needs its book and chapter".to_string()),
    }
}

/// A JSON field that may be written as a string or a number: "1" or 1.
fn number(field: &str, value: Option<Value>) -> Result<Option<String>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Number(n)) => Ok(Some(n.to_string())),
        Some(_) => Err(format!("{field} must be a number, like \"1\"")),
    }
}

/// The HTTP API's routes as a body: `books`, `books/1`, `books/1/1`,
/// `books/1/1/1`, with or without the leading `books` or `/`.
fn parse_path(body: &str) -> Option<Request> {
    let mut parts: Vec<&str> = body.split('/').filter(|p| !p.is_empty()).collect();
    if parts.first() == Some(&"books") {
        parts.remove(0);
    }
    if !parts.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    Some(match parts.as_slice() {
        [] => Request::Books { full: false },
        [b] => Request::Book(b.to_string()),
        [b, c] => Request::Chapter(b.to_string(), c.to_string()),
        [b, c, v] => Request::Verse(b.to_string(), c.to_string(), v.to_string()),
        _ => return None,
    })
}

fn answer(request: Request) -> Result<String, String> {
    match request {
        Request::Books { full: false } => {
            let books: Vec<Value> = scripture::books()
                .map(|b| json!({ "id": b.number().to_string(), "name": b.name() }))
                .collect();
            success(books)
        }
        Request::Books { full: true } => {
            success(scripture::books().map(scripture::BookRef::to_book).collect::<Vec<_>>())
        }
        Request::Book(b) => success(find_book(&b)?.to_book()),
        Request::Chapter(b, c) => success(find_chapter(&b, &c)?.to_chapter()),
        Request::Verse(b, c, v) => {
            let verse = find_chapter(&b, &c)?
                .find_verse(&v)
                .ok_or_else(|| format!("Verse {v} not found in chapter {c} of book '{b}'"))?;
            success(verse.to_verse())
        }
        Request::Search(query) => success(scripture::search(&query)),
    }
}

fn find_book(b: &str) -> Result<scripture::BookRef, String> {
    scripture::find_book(b).ok_or_else(|| format!("Book '{b}' not found"))
}

fn find_chapter(b: &str, c: &str) -> Result<scripture::ChapterRef, String> {
    find_book(b)?
        .find_chapter(c)
        .ok_or_else(|| format!("Chapter {c} not found in book '{b}'"))
}

fn success<T: Serialize>(data: T) -> Result<String, String> {
    serde_json::to_string(&BibleResponse { success: true, data }).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(body: &str) -> Value {
        serde_json::from_str(&handle(body)).expect("the answer is JSON")
    }

    #[test]
    fn an_empty_body_lists_the_books() {
        for body in ["", "  \n", "books", "/books/", "{}"] {
            let v = call(body);
            assert_eq!(v["success"], true, "{body:?}");
            assert_eq!(v["data"].as_array().unwrap().len(), 66, "{body:?}");
            assert_eq!(v["data"][0], json!({ "id": "1", "name": "Mwanzo" }), "{body:?}");
        }
    }

    #[test]
    fn paths_and_json_say_the_same() {
        let verse = json!({ "verse_number": "1", "verse_text": "Hapo mwanzo Mungu aliziumba mbingu na nchi. " });
        for body in ["books/1/1/1", "/books/1/1/1", "1/1/1", r#"{"book":"1","chapter":"1","verse":"1"}"#, r#"{"book":1,"chapter":1,"verse":1}"#] {
            assert_eq!(call(body), json!({ "success": true, "data": verse }), "{body:?}");
        }

        let chapter = call("books/66/22");
        assert_eq!(chapter["data"]["chapter_number"], "22");
        assert_eq!(chapter["data"]["VERSES"].as_array().unwrap().len(), 21);
        assert_eq!(chapter, call(r#"{"book":"66","chapter":"22"}"#));

        let book = call("books/66");
        assert_eq!(book["data"]["book_number"], "66");
        assert_eq!(book["data"]["book_name"], "Ufunuo wa Yohana");
        assert_eq!(book["data"]["CHAPTER"].as_array().unwrap().len(), 22);
        assert_eq!(book, call(r#"{"book":"66"}"#));
    }

    #[test]
    fn full_is_the_whole_bible() {
        let v = call(r#"{"full":true}"#);
        let books = v["data"].as_array().unwrap();
        assert_eq!(books.len(), 66);
        assert_eq!(books[65]["CHAPTER"][21]["VERSES"][20]["verse_number"], "21");
    }

    #[test]
    fn other_text_searches() {
        for body in ["Hapo mwanzo", r#"{"query":"Hapo mwanzo"}"#] {
            let v = call(body);
            assert_eq!(v["success"], true);
            let hits = v["data"].as_array().unwrap();
            assert_eq!(hits[0], json!({ "book": "Mwanzo", "chapter": "1", "verse": "1", "text": "Hapo mwanzo Mungu aliziumba mbingu na nchi. " }));
        }
        assert_eq!(call("books/not-a-number")["data"], json!([]));
    }

    #[test]
    fn says_what_was_not_found() {
        assert_eq!(call("books/67"), json!({ "success": false, "error": "Book '67' not found" }));
        assert_eq!(call("books/1/51"), json!({ "success": false, "error": "Chapter 51 not found in book '1'" }));
        assert_eq!(call("books/1/1/32"), json!({ "success": false, "error": "Verse 32 not found in chapter 1 of book '1'" }));
        assert_eq!(call("1/2/3/4")["data"], json!([]), "four numbers are a search, not a route");
    }

    #[test]
    fn refuses_bad_json() {
        let v = call(r#"{"book":"#);
        assert_eq!(v["success"], false);
        assert!(v["error"].as_str().unwrap().starts_with("the body must be JSON"));

        let v = call(r#"{"books":"1"}"#);
        assert_eq!(v["success"], false);
        assert!(v["error"].as_str().unwrap().contains("unknown field"));

        let v = call(r#"{"book":"1","verse":"1"}"#);
        assert_eq!(v["error"], "a chapter needs its book, and a verse needs its book and chapter");

        let v = call(r#"{"book":true}"#);
        assert_eq!(v["error"], "book must be a number, like \"1\"");
    }
}
