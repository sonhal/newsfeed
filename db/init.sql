-- -- Create a dedicated user for the application
-- CREATE USER newsfeed_user WITH PASSWORD 'user_password';
-- ALTER USER newsfeed_user WITH SUPERUSER; -- Optional, but useful for development

-- -- Create the database
-- CREATE DATABASE newsfeed_db OWNER newsfeed_user;

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE newsfeed_db TO newsfeed_user;

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE subscriptions (
    id SERIAL PRIMARY KEY,
    label TEXT NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('webpage', 'rss', 'api', 'atom')), -- Defines how to handle it
    polling_interval INTEGER NOT NULL CHECK (polling_interval > 0), -- In seconds/minutes
    last_checked TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE (user_id, url) -- Prevents duplicate subscriptions per user
);

CREATE TABLE articles (
    id SERIAL PRIMARY KEY,
    subscription_id INTEGER NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE, -- Foreign key from subscriptions
    title TEXT NOT NULL,  -- Title of the entry (RSS item, Tweet, etc.)
    content TEXT,         -- Full content of the post/item (can be NULL)
    source_url TEXT,      -- URL to the source item (optional, for reference)
    unique_identifier TEXT NOT NULL, -- Unique ID for deduplication (e.g., RSS GUID, Tweet ID, etc.)
    published_at TIMESTAMP WITH TIME ZONE, -- Date and time the item was published
    fetched_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(), -- Timestamp when the item was fetched
    data JSONB,       -- Any additional metadata (e.g., author, tags, etc.)
    UNIQUE (subscription_id, unique_identifier) -- Ensures deduplication per subscription
);

insert into users (username, email) values ('sondre', 'sondre@sonhal.no');