use axum::{
    extract::Extension,
    routing::{post},
    Router,
};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber;
use crate::handlers::{register, login, conversation_handler};
use crate::models::AppState;
use tower_http::cors::{CorsLayer, Any};
mod handlers;
mod models;
mod schema;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载环境变量
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // 创建数据库连接池
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let pool = Arc::new(pool);


    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)  // Allows requests from any origin (adjust this as needed)
        .allow_methods(Any) // Allow all HTTP methods
        .allow_headers(Any); // Allow all headers

    // 创建广播通道用于消息广播
    let (tx, _) = tokio::sync::broadcast::channel::<String>(100);

    // 创建共享的应用程序状态
    let app_state = AppState::new(tx.clone());

    // 构建应用路由
    let app = Router::new()
        .route("/register", post(register))  // 注册接口
        .route("/login", post(login))        // 登录接口
        .route("/conversation", post(conversation_handler))  // 处理对话的接口
        .layer(Extension(pool))
        .layer(Extension(app_state.clone()))
        .layer(cors);

    // 创建 TCP 监听器并启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);

    // 启动服务器，使用 axum::serve
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
