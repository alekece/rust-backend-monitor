CREATE TABLE status(
       id SERIAL PRIMARY KEY,
       created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
       monitor_id BIGINT UNSIGNED NOT NULL REFERENCES monitors(id) ON DELETE CASCADE,
       succeed BOOLEAN NOT NULL,
       response_time_ms INT UNSIGNED NOT NULL,
       result VARCHAR(255) NOT NULL
)
