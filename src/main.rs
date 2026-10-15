// One crate, two programs.
//
// On a real OS this is the HTTP server (axum + Postgres, `server.rs`). Built
// for wasm32-wasip1 it is a Barua Function: stdin in, stdout out, no network,
// no filesystem (`function.rs`). Both answer from the Bible compiled in by
// build.rs (`scripture.rs`).

mod function;
mod models;
mod scripture;

#[cfg(not(target_os = "wasi"))]
mod routes;
#[cfg(not(target_os = "wasi"))]
mod server;
#[cfg(not(target_os = "wasi"))]
mod utils;

#[cfg(target_os = "wasi")]
fn main() {
    function::run();
}

#[cfg(not(target_os = "wasi"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::run().await
}
