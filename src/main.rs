mod errors;
mod models;
mod storage;
mod handlers;

use actix_web::{middleware, web, App, HttpServer};
use std::path::PathBuf;
use storage::Storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Blog API server...");

    // Initialize storage
    let storage = Storage::new(PathBuf::from("data/blog.json"))
        .expect("Failed to initialize storage");

    println!("Server running at http://localhost:8080");
    println!("\nAvailable endpoints:");
    println!("  GET    /posts");
    println!("  GET    /posts/{{id}}");
    println!("  POST   /posts");
    println!("  PUT    /posts/{{id}}");
    println!("  DELETE /posts/{{id}}");
    println!("  GET    /posts/{{post_id}}/comments");
    println!("  POST   /posts/{{post_id}}/comments");
    println!("  DELETE /comments/{{id}}");
    println!("\nPress Ctrl+C to stop");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(storage.clone()))
            .wrap(middleware::Logger::default())
            .service(handlers::posts::get_posts)
            .service(handlers::posts::get_post)
            .service(handlers::posts::create_post)
            .service(handlers::posts::update_post)
            .service(handlers::posts::delete_post)
            .service(handlers::comments::get_comments)
            .service(handlers::comments::create_comment)
            .service(handlers::comments::delete_comment)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
