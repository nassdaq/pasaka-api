

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


