

# Pasaka Swahili Bible API

This is a blazing-fast Swahili Bible API built for developers who need scripture access in milliseconds. Whether you're working on a mobile app, website, or chatbot, the Pasaka API delivers Bible content quickly and efficiently.

## Features

* Ultra-fast RESTful API built with **Rust** and **Axum**
* Access to **all books**, **chapters**, and **verses** from the Swahili Bible
* Powerful **search** by keyword
* API Key secured endpoints
* Swagger UI documentation

## Testing the API (Swagger)

You can explore all endpoints via Swagger. To authorize:

1. Click the "Authorize" button
2. Enter your API key in this format:

```
x-api-key: your_api_key_here
```

## Using the API via `curl`

Example:

```bash
curl -H "x-api-key: pasaka_api_7a782fcd-da06-455xxxxxx" \
https://your-api-domain.com/books/66/1/1
```

**Response**:

```json
{
  "data": {
    "verse_number": "1",
    "verse_text": "Ufunuo wa Yesu Kristo, aliopewa na Mungu awaonyeshe watumwa wake mambo ambayo kwamba hayana budi kuwako upesi..."
  },
  "success": true
}
```

## Endpoints

| Method | Endpoint                             | Description                    |
| ------ | ------------------------------------ | ------------------------------ |
| GET    | `/books`                             | Get Entire Bible               |
| GET    | `/books/{book_id}`                   | Get info about a specific book |
| GET    | `/books/{book_id}/{chapter}`         | Get a chapter from a book      |
| GET    | `/books/{book_id}/{chapter}/{verse}` | Get a specific verse           |
| GET    | `/search?query={text}`               | Search verses by keyword       |
| POST   | `/auth/register`                     | Register new user              |
| POST   | `/auth/login`                        | Login & get token              |

## Authentication

All Bible data routes are protected by API Key. Add header in requests:

```http
x-api-key: your_api_key
```

You’ll receive your API key upon registration or request one manually.

## Running it as a Barua Function

The same repository deploys to [Barua Functions](https://fn.barua.app): paste
`nassdaq/pasaka-api`, and it is compiled to WebAssembly (`wasm32-wasip1`) with
the whole Bible built in. A function has no network and no filesystem, so there
is no Postgres, no registration and no API key there: the public URL is the API,
and a request is just the body you send it.

| Body | Returns |
| ---------------------------------------------- | ------------------------------------ |
| *(empty)*                                      | all books, id and name               |
| `books/1`                                      | a book with all its chapters         |
| `books/1/1`                                    | a chapter                            |
| `books/66/1/1`                                 | a verse                              |
| `{"book":"66","chapter":"1","verse":"1"}`      | the same, as JSON                    |
| `{"query":"Mungu ni upendo"}` or any other text | search                              |
| `{"full":true}`                                | every book with every verse (5 MB)   |

```bash
curl https://fn.barua.app/pasaka-api -d books/66/1/1
curl https://fn.barua.app/pasaka-api -d 'Mungu ni upendo'
```

Answers have the same shape as the HTTP API: `{"success":true,"data":…}` or
`{"success":false,"error":"…"}`.

To build the module yourself:

```bash
cargo build --release --target wasm32-wasip1
# target/wasm32-wasip1/release/pasaka-api.wasm
```

## How the data is stored

`build.rs` turns `src/data/swahili_bible.json` into a compile-time index (one
string of verse text plus offset tables), so both the server and the function
carry the Bible inside the binary and read nothing at run time. `cargo test`
checks the lookups and the function's request handling.

## Credits

Full credit to contributors who provided clean, usable Swahili Bible JSON data, enabling this API to exist.

## Tech Stack

* **Rust**
* **Axum**
* **API Key Security**
* **Swagger + Utoipa**
* **Tests included**
* **Fast JSON search**

## Contributing

PRs, ideas, and contributions are welcome:

1. Fork the repo
2. Clone and build with `cargo run`
3. Make your changes
4. Open a pull request

## Licence

This project is open-source and available under the **Jesus License**.


