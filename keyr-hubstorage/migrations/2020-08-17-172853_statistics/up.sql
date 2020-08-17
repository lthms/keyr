-- Your SQL goes here
CREATE TABLE statistics (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    count INTEGER NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id),

    CONSTRAINT unique_timestamp UNIQUE (timestamp, user_id)
)
