use axum::{debug_handler, extract::{Json, Extension}, http::StatusCode, response::IntoResponse};
use axum_extra::extract::TypedHeader;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use headers::Authorization;
use headers::authorization::Bearer;
use uuid::Uuid;
use rand::seq::SliceRandom;
use reqwest::Client;
use crate::DbPool;
use crate::models::{AppState, NewMessage, NewUser, User};
use crate::schema::users::dsl::*;
use crate::schema::messages::dsl::*;
use crate::schema::users::dsl::{users, id as user_id};
use crate::schema::messages::dsl::{messages, id as message_id};

/// 通用API响应结构
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,   // 用于成功时返回的数据
    pub message: Option<String>,  // 用于错误消息或提示信息
}

/// 注册请求结构
#[derive(Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub password: String,
    pub user_type: String, // "real" or "ai"
    pub ai_profile: Option<Value>, // Optional for AI users
}

/// 登录请求结构
#[derive(Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

/// 注册功能
#[debug_handler]
pub async fn register(
    Extension(pool): Extension<Arc<DbPool>>,
    Json(payload): Json<RegisterInput>,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 检查用户名是否已存在
    let existing_user = users
        .filter(username.eq(&payload.username))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(_) = existing_user {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Username already exists".to_string()),
        }));
    }

    // 对密码进行哈希处理并保存
    let hashed_password = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let new_user = NewUser {
        username: &payload.username,
        password_hash: &hashed_password,
        user_type: &payload.user_type,
        ai_profile: payload.ai_profile.as_ref(),
        session_token: None,
    };

    let created_user = diesel::insert_into(users)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(created_user),
        message: None,
    }))
}

/// 登录功能
#[debug_handler]
pub async fn login(
    Extension(pool): Extension<Arc<DbPool>>,
    Json(payload): Json<LoginInput>,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 查找用户
    let user = users
        .filter(username.eq(&payload.username))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if user.is_none() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("User not found".to_string()),
        }));
    }

    let mut user = user.unwrap();

    // 比对明文密码与哈希密码
    if verify(&payload.password, &user.password_hash).is_err() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Incorrect password".to_string()),
        }));
    }

    // 生成 session token 并更新到数据库
    let new_session_token = Uuid::new_v4().to_string();
    println!("new session token: {}", &new_session_token);

    diesel::update(users)
        .filter(user_id.eq(user.id))
        .set(session_token.eq(&new_session_token))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 更新 user 对象中的 session_token
    user.session_token = Some(new_session_token);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(user),  // 返回包含新 session_token 的用户信息
        message: None,
    }))
}

#[derive(Deserialize)]
pub struct MessagePayload {
    content: String,
}

/// 处理对话请求的功能

#[debug_handler]
pub async fn conversation_handler(
    Extension(pool): Extension<Arc<DbPool>>,
    Extension(app_state): Extension<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,  // Correct way to extract the Bearer token
    Json(payload): Json<MessagePayload>,  // 使用结构体接收带有 content 字段的 JSON 对象
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let token = bearer.token();  // 从 Bearer authorization 中提取 token
    println!("Received session token: {}", token);

    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 查询数据库中的所有用户以调试
    match users.load::<User>(&mut conn) {
        Ok(all_users) => {
            println!("All users in database: {:?}", all_users);
        },
        Err(e) => {
            println!("Failed to fetch users: {}", e);
        }
    }

    // 根据 session_token 查找用户
    match users.filter(session_token.eq(token)).first::<User>(&mut conn) {
        Ok(user) => {
            println!("Found user: {:?}", user);
            // 判断用户类型是 "real" 还是 "ai"
            let is_real_user = user.user_type == "real";

            // 将用户输入的消息存储
            let new_message = NewMessage {
                user_id: Some(user.id),
                content: Some(&payload.content),  // 使用结构体中的 content 字段
                image_url: None,
            };
            diesel::insert_into(messages)
                .values(&new_message)
                .execute(&mut conn)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if is_real_user {
                // 随机选取一个 AI 用户
                let ai_users: Vec<User> = users
                    .filter(user_type.eq("ai"))
                    .load(&mut conn)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                if ai_users.is_empty() {
                    return Ok(Json(ApiResponse {
                        success: false,
                        data: None,
                        message: Some("No AI users available".to_string()),
                    }));
                }

                let ai_user = ai_users.choose(&mut rand::thread_rng()).unwrap();
                let ai_response = call_ollama_api(ai_user, payload.content.clone())
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                // 将 AI 回复存储
                let new_message = NewMessage {
                    user_id: Some(ai_user.id),
                    content: Some(&ai_response),
                    image_url: None,
                };
                diesel::insert_into(messages)
                    .values(&new_message)
                    .execute(&mut conn)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                return Ok(Json(ApiResponse {
                    success: true,
                    data: Some(ai_response),
                    message: None,
                }));
            }

            Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("AI users cannot start a conversation.".to_string()),
            }))
        },
        Err(e) => {
            println!("Failed to find user with session token {}: {}", token, e);
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("Invalid session token".to_string()),
            }))
        }
    }
}


/// 调用 AI 服务的函数
async fn call_ollama_api(ai_user: &User, user_input: String) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let prompt = format!("AI Profile: {:?}\nTopic: {}", ai_user.ai_profile, user_input);

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&json!({ "model": "qwen2.5-coder", "prompt": prompt }))
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(response["message"]["content"]
        .as_str()
        .unwrap_or("AI Error")
        .to_string())
}
