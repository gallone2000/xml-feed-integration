CREATE TABLE posts (
    id         BIGSERIAL    PRIMARY KEY,
    title      TEXT         NOT NULL,
    url        TEXT         NOT NULL UNIQUE,
    img_url    TEXT,
    body       TEXT         NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_posts_url ON posts (url);
