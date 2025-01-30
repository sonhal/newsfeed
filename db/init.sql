-- -- Create a dedicated user for the application
-- CREATE USER newsfeed_user WITH PASSWORD 'user_password';
-- ALTER USER newsfeed_user WITH SUPERUSER; -- Optional, but useful for development

-- -- Create the database
-- CREATE DATABASE newsfeed_db OWNER newsfeed_user;

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE newsfeed_db TO newsfeed_user;


CREATE TABLE subscribed (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);