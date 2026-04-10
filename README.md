# xml-feed-fetcher

A Rust CLI tool that fetches posts from an XML/Atom feed (e.g. a Blogger blog), extracts the title, cover image, and plain-text body of each post, and persists them into a PostgreSQL database.

## How it works

1. Reads `FEED_URL` and `DATABASE_URL` from a `.env` file.
2. Paginates through the feed, fetching posts in parallel (up to 10 concurrent requests).
3. For each post, scrapes the full page to extract the `og:title`, `og:image`, and the `div.post-body` content (converted to clean plain text).
4. Inserts new posts into the `posts` table; already-present posts are silently skipped.
5. Errors are logged to `logs/errors.log`.

## Requirements

- [Rust](https://rustup.rs/) (stable)
- [Docker](https://www.docker.com/) (for the PostgreSQL container)
- `sqlx-cli` (installed once with `make install-tools`)

## Setup

Copy `.env.example` to `.env` (or create `.env`) and fill in your values:

```env
POSTGRES_DB=feeddb
POSTGRES_USER=feeduser
POSTGRES_PASSWORD=feedpass
DATABASE_URL=postgres://feeduser:feedpass@localhost:5437/feeddb
FEED_URL=https://yourblog.blogspot.com/feeds/posts/default?alt=atom
```

Install `sqlx-cli`, start the database, and apply migrations in one step:

```bash
make install-tools   # only needed once
make init            # starts Docker Postgres + runs migrations
```

## Running

```bash
make fetch           # fetch all posts from the feed and save them to the DB
```

## Other useful commands

| Command              | Description                                      |
|----------------------|--------------------------------------------------|
| `make build`         | Optimised release build                          |
| `make check`         | Check for compile errors without building        |
| `make fmt`           | Format code with `rustfmt`                       |
| `make lint`          | Lint with `clippy`                               |
| `make test`          | Run tests (Postgres is started automatically via testcontainers) |
| `make db-up`         | Start the PostgreSQL Docker container            |
| `make db-down`       | Stop and remove the PostgreSQL container         |
| `make migrate`       | Apply pending migrations                         |
| `make migrate-down`  | Revert the last migration                        |
| `make db-reset`      | Wipe all data and re-initialise from scratch     |
| `make sqlx-prepare`  | Regenerate `.sqlx/` cache for offline builds     |
