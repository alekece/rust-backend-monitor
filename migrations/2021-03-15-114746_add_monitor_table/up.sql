CREATE TABLE monitors(
       id SERIAL PRIMARY KEY,
       type VARCHAR(255) NOT NULL,
       frequency_min SMALLINT UNSIGNED NOT NULL,
       endpoint VARCHAR(255) NOT NULL,
       max_latency_ms INT UNSIGNED,
       expected_ip VARCHAR(255),
       minimum_expiration_time_d INT UNSIGNED,
       UNIQUE (type, endpoint)
);
