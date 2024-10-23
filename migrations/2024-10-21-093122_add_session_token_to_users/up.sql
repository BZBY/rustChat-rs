-- Your SQL goes here
-- 新增 session_token 字段到 users 表

ALTER TABLE users ALTER COLUMN session_token TYPE VARCHAR;
ALTER TABLE users ALTER COLUMN session_token DROP NOT NULL;
