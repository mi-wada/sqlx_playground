CREATE DATABASE db;

\c db;

CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL,
  note VARCHAR(255),
  is_active BOOLEAN DEFAULT TRUE
);
