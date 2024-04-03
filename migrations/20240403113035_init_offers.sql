CREATE TABLE offers (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    location TEXT,
    rooms SMALLINT,
    area REAL,
    detail_url TEXT,
    price INTEGER
);