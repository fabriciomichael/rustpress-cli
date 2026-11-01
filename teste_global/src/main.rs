mod routes;
mod controllers;
mod models;
mod middleware;
mod config;

use rustpress_web::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = App::new();

    routes::web::register(&mut app)?;

    println!("RustPress running at http://127.0.0.1:3000");

    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
