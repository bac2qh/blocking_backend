use crate::errors::{ApiError, ApiResult};
use crate::models::{Comment, Post, UpdatePost};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;
use std::collections::HashMap;
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

        let storage = Storage {
            data: Arc::new(Mutex::new(data)),
            file_path,
        };

          // Save initial data if file doesn't exist
        if !storage.file_path.exists() {
            storage.save()?;
        }

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
    pub fn get_all_posts(&self) -> ApiResult<HashMap<Uuid, Post>> {
        let data = self.data.lock().unwrap();
        Ok(data.posts.clone())
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
        post.update(UpdatePost { title, content })?;
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
        let mut data = self.data.lock().unwrap();
        data.comments.push(comment.clone());
        drop(data);
        self.save()?;
        Ok(comment)
    }

    pub fn delete_comment(&self, id: Uuid) -> ApiResult<()> {
        let mut data = self.data.lock().unwrap();
        let index = data.comments.iter().position(|c| c.id == id).ok_or(ApiError::CommentNotFound)?;

        data.comments.remove(index);
        drop(data);
        self.save()?;
        Ok(())
    }
}
