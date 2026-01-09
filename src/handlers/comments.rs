use crate::errors::{ApiError, ApiResult};
use crate::models::{Comment, CreateComment};
use crate::storage::Storage;
use actix_web::{delete, get, post, web, HttpResponse};
use uuid::Uuid;

#[get("/posts/{post_id}/comments")]
pub async fn get_comments(storage: web::Data<Storage>, post_id: web::Path<Uuid>) -> ApiResult<HttpResponse> {
    let comments = storage.get_post_comments(*post_id)?;
    Ok(HttpResponse::Ok().json(comments))
}

#[post("/posts/{post_id}/comments")]
pub async fn create_comment(storage: web::Data<Storage>, post_id: web::Path<Uuid>, new_comment: web::Json<CreateComment>) -> ApiResult<HttpResponse> {
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
pub async fn delete_comment(storage: web::Data<Storage>, id: web::Path<Uuid>) -> ApiResult<HttpResponse> {
    storage.delete_comment(*id)?;
    Ok(HttpResponse::NoContent().finish())
}