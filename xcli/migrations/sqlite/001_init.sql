CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
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
    path TEXT NOT NULL UNIQUE,
    summary TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cache_profiles (
    id TEXT PRIMARY KEY,
    handle TEXT NOT NULL,
    display_name TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cache_posts (
    id TEXT PRIMARY KEY,
    author TEXT NOT NULL,
    body TEXT NOT NULL,
    audience TEXT NOT NULL,
    created_at TEXT NOT NULL,
    author_handle TEXT,
    liked INTEGER NOT NULL DEFAULT 0,
    like_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS cache_follows (
    follower TEXT NOT NULL,
    followee TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (follower, followee)
);

CREATE TABLE IF NOT EXISTS cache_likes (
    user_id TEXT NOT NULL,
    post_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (user_id, post_id)
);

CREATE TABLE IF NOT EXISTS outbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
);
