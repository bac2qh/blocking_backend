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
pub async fn get_post(storage: web::Data<Storage>, id: web::Path<Uuid>) -> ApiResult<HttpResponse> {
    let post = storage.get_post(*id)?;
    let comments = storage.get_post_comments(*id)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "post": post,
        "comments": comments
    })))
}

#[post("/posts")]
pub async fn create_post(storage: web::Data<Storage>, new_post: web::Json<CreatePost>) -> ApiResult<HttpResponse> {
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
pub async fn update_post(storage: web::Data<Storage>, id: web::Path<Uuid>, update_post: web::Json<UpdatePost>) -> ApiResult<HttpResponse> {
    // Validation
    if let Some(title) = update_post.title.as_ref() {
        if title.trim().is_empty() {
            return Err(ApiError::ValidationError("Title cannot be empty".to_string()));
        }
        if title.len() > 200 {
            return Err(ApiError::ValidationError("Title too long (max 200 characters)".to_string()));
        }
    }
    if let Some(content) = update_post.content.as_ref() {
        if content.trim().is_empty() {
            return Err(ApiError::ValidationError("Content cannot be empty".to_string()));
        }
    }

    let updated_post = storage.update_post(*id, update_post.title.clone(), update_post.content.clone())?;
    Ok(HttpResponse::Ok().json(updated_post))
}

#[delete("/posts/{id}")]
pub async fn delete_post(storage: web::Data<Storage>, id: web::Path<Uuid>) -> ApiResult<HttpResponse> {
    storage.delete_post(*id)?;
    Ok(HttpResponse::NoContent().finish())
}