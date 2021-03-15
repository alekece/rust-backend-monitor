CREATE TABLE external_monitors(
       id SERIAL PRIMARY KEY,
       created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
       serial VARCHAR(255) NOT NULL,
       cpu_usage TINYINT UNSIGNED NOT NULL,
       memory_usage TINYINT UNSIGNED NOT NULL,
       disk_usage TINYINT UNSIGNED NOT NULL,
       status TEXT
);
