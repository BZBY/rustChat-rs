CREATE TABLE messages (
                          id SERIAL PRIMARY KEY,
                          user_id INTEGER REFERENCES users(id),
                          content TEXT,
                          image_url VARCHAR,
                          created_at TIMESTAMP NOT NULL DEFAULT NOW()
);