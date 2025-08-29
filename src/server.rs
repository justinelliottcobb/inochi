use actix_files as fs;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../www/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://localhost:3000");
    println!("Press Ctrl+C to stop the server");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .header("Cross-Origin-Embedder-Policy", "require-corp")
                    .header("Cross-Origin-Opener-Policy", "same-origin"),
            )
            .route("/", web::get().to(index))
            .service(fs::Files::new("/", "./www").index_file("index.html"))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}