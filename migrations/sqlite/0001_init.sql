CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT
);

CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('open','done')) DEFAULT 'open',
    created_at TEXT NOT NULL,
    done_at TEXT
);

CREATE TABLE IF NOT EXISTS reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER,
    path TEXT UNIQUE NOT NULL,
    summary TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cache_profiles (
    id TEXT PRIMARY KEY,
    handle TEXT,
    display_name TEXT,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS cache_posts (
    id TEXT PRIMARY KEY,
    author TEXT,
    body TEXT,
    audience TEXT,
    created_at TEXT,
    author_handle TEXT,
    like_count INTEGER
);

CREATE TABLE IF NOT EXISTS cache_follows (
    follower TEXT,
    followee TEXT,
    created_at TEXT,
    PRIMARY KEY(follower, followee)
);

CREATE TABLE IF NOT EXISTS cache_likes (
    user_id TEXT,
    post_id TEXT,
    created_at TEXT,
    PRIMARY KEY(user_id, post_id)
);

CREATE TABLE IF NOT EXISTS outbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0
);
