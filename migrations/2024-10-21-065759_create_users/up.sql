CREATE TABLE users (
                       id SERIAL PRIMARY KEY,
                       username VARCHAR NOT NULL UNIQUE,
                       password_hash VARCHAR NOT NULL,
                       user_type VARCHAR NOT NULL,  -- 'real' 或 'ai'
                       ai_profile JSONB,            -- AI 客户端的个人信息
                       session_token VARCHAR,       -- 新增的会话令牌字段
                       created_at TIMESTAMP NOT NULL DEFAULT NOW()
);