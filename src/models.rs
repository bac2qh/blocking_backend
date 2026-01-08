use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::errors::ApiResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub content: String,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)] 
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateComment {
    pub post_id: Uuid,
    pub content: String,
    pub author: String,
}

impl Post {
    pub fn new(create_post: CreatePost) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: create_post.title,
            content: create_post.content,
            author: create_post.author,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    pub fn update(&mut self, update_post: UpdatePost) -> ApiResult<()> {
        if let Some(title) = update_post.title {
            self.title = title;
        }
        if let Some(content) = update_post.content {
            self.content = content;
        }
        self.updated_at = Utc::now();
        Ok(())
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
            updated_at: Utc::now(),
        }
    }
}