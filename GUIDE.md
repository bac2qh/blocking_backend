# Building a Blocking REST API in Rust - Complete Guide

A step-by-step guide to building a simple blog API using Rust, Actix-web (blocking mode), and JSON file storage.

## Project Overview

**What we're building:** A REST API for a simple blog with posts and comments
**Storage:** JSON file (no database)
**Concurrency:** Multi-threaded blocking I/O (no async/await)
**Learning goals:** Rust fundamentals, ownership, error handling, web development basics

---

## Step 1: Project Setup

### 1.1 Create the project
```bash
cargo new blocking_backend
cd blocking_backend
```

### 1.2 Update Cargo.toml

Replace the contents with:

```toml
[package]
name = "blocking_backend"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = { version = "4", default-features = false, features = ["macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "1"
```

### 1.3 Download dependencies
```bash
cargo build
```

---

## Step 2: Data Models (src/models.rs)

Create `src/models.rs` with the following content:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub content: String,
    pub author: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateComment {
    pub post_id: Uuid,
    pub content: String,
    pub author: String,
}

impl Post {
    pub fn new(create_post: CreatePost) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: create_post.title,
            content: create_post.content,
            author: create_post.author,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, update_post: UpdatePost) {
        if let Some(title) = update_post.title {
            self.title = title;
        }
        if let Some(content) = update_post.content {
            self.content = content;
        }
        self.updated_at = Utc::now();
    }
}

impl Comment {
    pub fn new(create_comment: CreateComment) -> Self {
        Self {
            id: Uuid::new_v4(),
            post_id: create_comment.post_id,
            content: create_comment.content,
            author: create_comment.author,
            created_at: Utc::now(),
        }
    }
}
```

**Key concepts:**
- `#[derive(Serialize, Deserialize)]`: Enables JSON conversion
- `CreatePost` vs `Post`: Separates client input from server-managed data (id, timestamps)
- `Option<String>` in `UpdatePost`: Allows partial updates

---

## Step 3: Error Handling (src/errors.rs)

Create `src/errors.rs`:

```rust
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Post not found")]
    PostNotFound,

    #[error("Comment not found")]
    CommentNotFound,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Internal server error")]
    InternalError,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::PostNotFound | ApiError::CommentNotFound => StatusCode::NOT_FOUND,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::StorageError(_) | ApiError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": self.to_string()
        }))
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
```

**Key concepts:**
- `#[derive(Error)]`: thiserror macro generates error trait implementations
- `ResponseError`: Converts Rust errors to HTTP responses automatically
- `ApiResult<T>`: Type alias for `Result<T, ApiError>`

---

## Step 4: Storage Layer (src/storage.rs)

Create `src/storage.rs`:

```rust
use crate::errors::{ApiError, ApiResult};
use crate::models::{Comment, Post, UpdatePost};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogData {
    pub posts: HashMap<Uuid, Post>,
    pub comments: Vec<Comment>,
}

impl BlogData {
    pub fn new() -> Self {
        Self {
            posts: HashMap::new(),
            comments: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct Storage {
    data: Arc<Mutex<BlogData>>,
    file_path: PathBuf,
}

impl Storage {
    pub fn new(file_path: PathBuf) -> ApiResult<Self> {
        // Create data directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ApiError::StorageError(format!("Failed to create data directory: {}", e)))?;
        }

        // Load existing data or create new
        let data = if file_path.exists() {
            let contents = fs::read_to_string(&file_path)
                .map_err(|e| ApiError::StorageError(format!("Failed to read file: {}", e)))?;
            serde_json::from_str(&contents)
                .map_err(|e| ApiError::StorageError(format!("Failed to parse JSON: {}", e)))?
        } else {
            BlogData::new()
        };

        let storage = Self {
            data: Arc::new(Mutex::new(data)),
            file_path,
        };

        // Ensure file exists
        storage.save()?;

        Ok(storage)
    }

    fn save(&self) -> ApiResult<()> {
        let data = self.data.lock().unwrap();
        let json = serde_json::to_string_pretty(&*data)
            .map_err(|e| ApiError::StorageError(format!("Failed to serialize: {}", e)))?;

        let mut file = File::create(&self.file_path)
            .map_err(|e| ApiError::StorageError(format!("Failed to create file: {}", e)))?;

        file.write_all(json.as_bytes())
            .map_err(|e| ApiError::StorageError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    // Post operations
    pub fn get_all_posts(&self) -> ApiResult<Vec<Post>> {
        let data = self.data.lock().unwrap();
        Ok(data.posts.values().cloned().collect())
    }

    pub fn get_post(&self, id: Uuid) -> ApiResult<Post> {
        let data = self.data.lock().unwrap();
        data.posts.get(&id).cloned().ok_or(ApiError::PostNotFound)
    }

    pub fn create_post(&self, post: Post) -> ApiResult<Post> {
        let mut data = self.data.lock().unwrap();
        data.posts.insert(post.id, post.clone());
        drop(data);
        self.save()?;
        Ok(post)
    }

    pub fn update_post(&self, id: Uuid, title: Option<String>, content: Option<String>) -> ApiResult<Post> {
        let mut data = self.data.lock().unwrap();
        let post = data.posts.get_mut(&id).ok_or(ApiError::PostNotFound)?;
        post.update(UpdatePost { title, content });
        let updated_post = post.clone();
        drop(data);
        self.save()?;
        Ok(updated_post)
    }

    pub fn delete_post(&self, id: Uuid) -> ApiResult<()> {
        let mut data = self.data.lock().unwrap();
        data.posts.remove(&id).ok_or(ApiError::PostNotFound)?;
        // Remove all comments associated with the post
        data.comments.retain(|c| c.post_id != id);
        drop(data);
        self.save()?;
        Ok(())
    }

    // Comment operations
    pub fn get_post_comments(&self, post_id: Uuid) -> ApiResult<Vec<Comment>> {
        // Verify post exists
        self.get_post(post_id)?;

        let data = self.data.lock().unwrap();
        Ok(data.comments.iter()
            .filter(|c| c.post_id == post_id)
            .cloned()
            .collect())
    }

    pub fn create_comment(&self, comment: Comment) -> ApiResult<Comment> {
        // Verify post exists
        self.get_post(comment.post_id)?;

        let mut data = self.data.lock().unwrap();
        data.comments.push(comment.clone());
        drop(data);
        self.save()?;
        Ok(comment)
    }

    pub fn delete_comment(&self, id: Uuid) -> ApiResult<()> {
        let mut data = self.data.lock().unwrap();
        let index = data.comments.iter()
            .position(|c| c.id == id)
            .ok_or(ApiError::CommentNotFound)?;

        data.comments.remove(index);
        drop(data);
        self.save()?;
        Ok(())
    }
}
```

**Key concepts:**
- `Arc<Mutex<>>`: Thread-safe shared ownership
  - `Arc`: Multiple threads can own references to the data
  - `Mutex`: Only one thread can access the data at a time
- `HashMap` for posts: O(1) lookups by ID
- `Vec` for comments: Simple storage, filter when needed
- `drop(data)`: Explicitly release the lock before saving

---

## Step 5: HTTP Handlers

### 5.1 Create handlers directory structure

```bash
mkdir src/handlers
touch src/handlers/mod.rs
touch src/handlers/posts.rs
touch src/handlers/comments.rs
```

### 5.2 Create src/handlers/mod.rs

```rust
pub mod posts;
pub mod comments;
```

### 5.3 Create src/handlers/posts.rs

```rust
use crate::errors::{ApiError, ApiResult};
use crate::models::{CreatePost, Post, UpdatePost};
use crate::storage::Storage;
use actix_web::{delete, get, post, put, web, HttpResponse};
use uuid::Uuid;

#[get("/posts")]
pub async fn get_posts(storage: web::Data<Storage>) -> ApiResult<HttpResponse> {
    let posts = storage.get_all_posts()?;
    Ok(HttpResponse::Ok().json(posts))
}

#[get("/posts/{id}")]
pub async fn get_post(
    storage: web::Data<Storage>,
    id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let post = storage.get_post(*id)?;
    let comments = storage.get_post_comments(*id)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "post": post,
        "comments": comments
    })))
}

#[post("/posts")]
pub async fn create_post(
    storage: web::Data<Storage>,
    new_post: web::Json<CreatePost>,
) -> ApiResult<HttpResponse> {
    // Validation
    if new_post.title.trim().is_empty() {
        return Err(ApiError::ValidationError("Title cannot be empty".to_string()));
    }
    if new_post.content.trim().is_empty() {
        return Err(ApiError::ValidationError("Content cannot be empty".to_string()));
    }
    if new_post.author.trim().is_empty() {
        return Err(ApiError::ValidationError("Author cannot be empty".to_string()));
    }
    if new_post.title.len() > 200 {
        return Err(ApiError::ValidationError("Title too long (max 200 characters)".to_string()));
    }

    let post = Post::new(new_post.into_inner());
    let created_post = storage.create_post(post)?;
    Ok(HttpResponse::Created().json(created_post))
}

#[put("/posts/{id}")]
pub async fn update_post(
    storage: web::Data<Storage>,
    id: web::Path<Uuid>,
    update: web::Json<UpdatePost>,
) -> ApiResult<HttpResponse> {
    // Validation
    if let Some(ref title) = update.title {
        if title.trim().is_empty() {
            return Err(ApiError::ValidationError("Title cannot be empty".to_string()));
        }
        if title.len() > 200 {
            return Err(ApiError::ValidationError("Title too long (max 200 characters)".to_string()));
        }
    }
    if let Some(ref content) = update.content {
        if content.trim().is_empty() {
            return Err(ApiError::ValidationError("Content cannot be empty".to_string()));
        }
    }

    let updated_post = storage.update_post(*id, update.title.clone(), update.content.clone())?;
    Ok(HttpResponse::Ok().json(updated_post))
}

#[delete("/posts/{id}")]
pub async fn delete_post(
    storage: web::Data<Storage>,
    id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    storage.delete_post(*id)?;
    Ok(HttpResponse::NoContent().finish())
}
```

**Key concepts:**
- `#[get("/posts")]`: Actix-web routing macros
- `web::Data<Storage>`: Shared application state (Arc clone)
- `web::Json<T>`: Automatic JSON deserialization
- `async fn`: Required by Actix-web even in blocking mode
- `?` operator: Propagates errors (automatically converted to HTTP responses)

### 5.4 Create src/handlers/comments.rs

```rust
use crate::errors::{ApiError, ApiResult};
use crate::models::{Comment, CreateComment};
use crate::storage::Storage;
use actix_web::{delete, get, post, web, HttpResponse};
use uuid::Uuid;

#[get("/posts/{post_id}/comments")]
pub async fn get_comments(
    storage: web::Data<Storage>,
    post_id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let comments = storage.get_post_comments(*post_id)?;
    Ok(HttpResponse::Ok().json(comments))
}

#[post("/posts/{post_id}/comments")]
pub async fn create_comment(
    storage: web::Data<Storage>,
    post_id: web::Path<Uuid>,
    new_comment: web::Json<CreateComment>,
) -> ApiResult<HttpResponse> {
    // Validation
    if new_comment.author.trim().is_empty() {
        return Err(ApiError::ValidationError("Author cannot be empty".to_string()));
    }
    if new_comment.content.trim().is_empty() {
        return Err(ApiError::ValidationError("Content cannot be empty".to_string()));
    }

    let comment = Comment::new(CreateComment {
        post_id: *post_id,
        author: new_comment.author.clone(),
        content: new_comment.content.clone(),
    });

    let created_comment = storage.create_comment(comment)?;
    Ok(HttpResponse::Created().json(created_comment))
}

#[delete("/comments/{id}")]
pub async fn delete_comment(
    storage: web::Data<Storage>,
    id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    storage.delete_comment(*id)?;
    Ok(HttpResponse::NoContent().finish())
}
```

---

## Step 6: Main Server Setup (src/main.rs)

Replace the contents of `src/main.rs`:

```rust
mod errors;
mod handlers;
mod models;
mod storage;

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
```

**Key concepts:**
- `#[actix_web::main]`: Sets up the runtime
- `HttpServer::new(move || ...)`: Creates server with app factory
- `.app_data()`: Shares storage across all threads
- `.service()`: Registers handler functions

---

## Step 7: Build and Run

### 7.1 Build the project
```bash
cargo build
```

### 7.2 Run the server
```bash
cargo run
```

You should see:
```
Starting Blog API server...
Server running at http://localhost:8080
...
Press Ctrl+C to stop
```

---

## Step 8: Testing the API

### 8.1 Create a post

```bash
curl -X POST http://localhost:8080/posts \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My First Post",
    "content": "Hello, Rust!",
    "author": "Alice"
  }'
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "My First Post",
  "content": "Hello, Rust!",
  "author": "Alice",
  "created_at": "2026-01-08T...",
  "updated_at": "2026-01-08T..."
}
```

### 8.2 Get all posts

```bash
curl http://localhost:8080/posts
```

### 8.3 Get a specific post with comments

```bash
curl http://localhost:8080/posts/{post_id}
```

Replace `{post_id}` with the actual UUID from the create response.

### 8.4 Update a post

```bash
curl -X PUT http://localhost:8080/posts/{post_id} \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Title"
  }'
```

### 8.5 Add a comment

```bash
curl -X POST http://localhost:8080/posts/{post_id}/comments \
  -H "Content-Type: application/json" \
  -d '{
    "author": "Bob",
    "content": "Great post!"
  }'
```

### 8.6 Delete a post

```bash
curl -X DELETE http://localhost:8080/posts/{post_id}
```

### 8.7 Delete a comment

```bash
curl -X DELETE http://localhost:8080/comments/{comment_id}
```

---

## Step 9: Verify Data Persistence

1. Create some posts and comments
2. Stop the server (Ctrl+C)
3. Restart the server: `cargo run`
4. Query the API - your data should still be there!

Check the JSON file:
```bash
cat data/blog.json
```

---

## What You've Learned

### Rust Fundamentals
- **Ownership**: Moving data with `CreatePost`, borrowing with `&self`
- **Borrowing**: References in function parameters
- **Lifetimes**: Implicit in struct methods
- **Error Handling**: `Result<T, E>`, `?` operator, custom errors
- **Pattern Matching**: `match` in error handling, `if let Some`
- **Traits**: `Serialize`, `Deserialize`, `Error`, `ResponseError`
- **Generics**: `ApiResult<T>`, `Vec<T>`, `HashMap<K, V>`

### Concurrency
- **Arc**: Atomic reference counting for shared ownership
- **Mutex**: Mutual exclusion for thread-safe access
- **Multi-threading**: Actix-web's thread pool

### Web Development
- **HTTP Methods**: GET, POST, PUT, DELETE
- **Status Codes**: 200 OK, 201 Created, 204 No Content, 400 Bad Request, 404 Not Found
- **JSON**: Serialization/deserialization with serde
- **REST**: Resource-oriented API design
- **Validation**: Input checking before processing

### Data Structures
- **HashMap**: O(1) lookups for posts
- **Vec**: Sequential storage for comments
- **Trade-offs**: Memory vs speed, simplicity vs optimization

---

## Next Steps

### Easy Improvements
1. Add pagination to GET /posts
2. Add sorting (by date, title, etc.)
3. Add filtering/search
4. Return more detailed error messages
5. Add logging for debugging

### Intermediate Challenges
1. Convert to async with tokio
2. Replace JSON with SQLite database
3. Add authentication (API keys or JWT)
4. Add middleware for rate limiting
5. Add tests (unit tests, integration tests)

### Advanced Features
1. Add WebSocket support for real-time updates
2. Implement full-text search
3. Add image upload for posts
4. Deploy to production (Docker, cloud hosting)
5. Add monitoring and metrics

---

## Common Issues and Solutions

### Issue: "address already in use"
**Solution:** Port 8080 is occupied. Change the port in main.rs or kill the process:
```bash
lsof -ti:8080 | xargs kill
```

### Issue: "failed to read file" on startup
**Solution:** Ensure the data directory exists and has write permissions:
```bash
mkdir -p data
chmod 755 data
```

### Issue: Can't parse UUID in curl request
**Solution:** Copy the exact UUID from a previous response, including dashes

### Issue: "Post not found" when adding comment
**Solution:** Make sure the post_id in the URL matches an existing post

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│              HTTP Client (curl)                  │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│         Actix-web HTTP Server (Port 8080)       │
│  ┌───────────────────────────────────────────┐  │
│  │          Thread Pool (Workers)            │  │
│  │  Thread 1 │ Thread 2 │ ... │ Thread N    │  │
│  └───────────────────────────────────────────┘  │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│                   Handlers                       │
│  posts.rs (get_posts, create_post, etc.)        │
│  comments.rs (get_comments, create_comment)      │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│           Storage Layer (storage.rs)             │
│         Arc<Mutex<BlogData>>                     │
│  ┌───────────────────────────────────────────┐  │
│  │  posts: HashMap<Uuid, Post>               │  │
│  │  comments: Vec<Comment>                   │  │
│  └───────────────────────────────────────────┘  │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│           File System (data/blog.json)           │
└─────────────────────────────────────────────────┘
```

---

## Glossary

- **Arc**: Atomically Reference Counted pointer - enables shared ownership across threads
- **Mutex**: Mutual Exclusion lock - ensures only one thread accesses data at a time
- **Blocking I/O**: Operations that wait (block) until complete (vs async which yields control)
- **DTO**: Data Transfer Object - structs for API input/output (CreatePost, UpdatePost)
- **CRUD**: Create, Read, Update, Delete - basic data operations
- **REST**: Representational State Transfer - API design using HTTP methods and resources
- **Trait**: Rust's interface system - defines behavior types can implement
- **Derive macro**: Code generation via `#[derive(...)]` - automatically implements traits
- **Type alias**: Shorthand for complex types (`ApiResult<T>` = `Result<T, ApiError>`)

---

## Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Actix-web Documentation](https://actix.rs/)
- [Serde Documentation](https://serde.rs/)
- [REST API Best Practices](https://restfulapi.net/)

---

Happy coding! 🦀
