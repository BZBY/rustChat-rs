use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use crate::schema::{messages, users};
use serde_json::Value;
use diesel::prelude::*;
use chrono::NaiveDateTime;

// User model
#[derive(Debug, Queryable, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub user_type: String,
    pub ai_profile: Option<Value>,  // Use serde_json::Value for JSONB
    pub created_at: NaiveDateTime,
    pub session_token: Option<String>,
}

// NewUser struct for inserting new users
#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub user_type: &'a str,
    pub ai_profile: Option<&'a Value>,  // Reference serde_json::Value for JSONB insertion
    pub session_token: Option<&'a str>,
}

// Message model
#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct Message {
    pub id: i32,
    pub user_id: Option<i32>,
    pub content: Option<String>,
    pub image_url: Option<String>,
    pub created_at: NaiveDateTime,
}

// NewMessage struct for inserting new messages
#[derive(Debug, Insertable)]
#[table_name = "messages"]
pub struct NewMessage<'a> {
    pub user_id: Option<i32>,
    pub content: Option<&'a str>,
    pub image_url: Option<&'a str>,
}

// AppState to manage shared state for AI clients and broadcasting
#[derive(Debug, Clone)]  // 去除 Serialize 和 Deserialize
pub struct AppState {
    pub tx: tokio::sync::broadcast::Sender<String>,
    pub ai_clients: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<i32, tokio::sync::broadcast::Sender<String>>>>,
}

impl AppState {
    pub fn new(tx: tokio::sync::broadcast::Sender<String>) -> Self {
        Self {
            tx,
            ai_clients: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}
