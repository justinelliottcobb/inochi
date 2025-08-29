use actix_files as fs;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use std::env;

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../../www/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Get host and port from environment variables with defaults
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid number");
    
    println!("Starting server at http://{}:{}", host, port);
    println!("Serving files from ./www directory");
    println!("Press Ctrl+C to stop the server");
    println!();
    println!("Environment variables:");
    println!("  HOST={} (default: 0.0.0.0)", host);
    println!("  PORT={} (default: 3000)", port);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Cross-Origin-Embedder-Policy", "require-corp"))
                    .add(("Cross-Origin-Opener-Policy", "same-origin")),
            )
            .route("/", web::get().to(index))
            .service(fs::Files::new("/", "./www").index_file("index.html"))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}