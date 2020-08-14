-- Your SQL goes here
CREATE TABLE tokens (
    id SERIAL PRIMARY KEY,
    token VARCHAR(32) NOT NULL UNIQUE,
    user_id INTEGER NOT NULL REFERENCES users(id)
)
