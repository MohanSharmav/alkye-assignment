# Submission Guide

## Overview

This document explains how to run the Task Management API locally and how to prepare it for production submission.

The project is a Rust-based REST API that uses PostgreSQL, Redis, and JWT-based authentication with email-style 2FA.

## Prerequisites

- Rust 1.70+ installed
- Docker and Docker Compose installed
- `cargo` available on your PATH
- `sqlx-cli` installed if running migrations locally

## Environment Files

The repository includes a sample environment file:

- `.env.example`

Copy it locally before starting:

```bash
cp .env.example .env
```

## Local Setup

### 1. Start dependencies with Docker Compose

From `task_api`:

```bash
docker-compose up -d
```

This starts:

- PostgreSQL on `localhost:5432`
- Redis on `localhost:6379`

### 2. Configure local environment variables

Create `.env` or export these values:

```bash
export DATABASE_URL="postgres://postgres:password@localhost:5432/task_api"
export REDIS_URL="redis://127.0.0.1:6379"
export RUST_LOG=info
```

### 3. Run database migrations

Install `sqlx-cli` if needed:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Then run migrations:

```bash
sqlx migrate run --database-url "$DATABASE_URL"
```

### 4. Build and run the app

```bash
cargo build
cargo run
```

The API listens on `http://127.0.0.1:3000`.

### 5. Validate workflow locally

A full validation script is available:

```bash
./validate.sh
```

Manual validation can be done with these steps:

1. Seed users:
   ```bash
   curl -X POST http://127.0.0.1:3000/seed/users
   ```
2. Login as admin:
   ```bash
   curl -X POST http://127.0.0.1:3000/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"admin@example.com","password":"admin123"}'
   ```
3. Retrieve verification code:
   ```bash
   curl http://127.0.0.1:3000/dev/email-logs/latest
   ```
4. Verify 2FA and receive JWT:
   ```bash
   curl -X POST http://127.0.0.1:3000/auth/verify-2fa \
     -H "Content-Type: application/json" \
     -d '{"login_challenge_id":"<id>","code":"<code>"}'
   ```
5. Use the returned Bearer token for protected endpoints.

### 6. Run tests

Run the validation test directly:

```bash
cargo test --test validation_flow -- --nocapture
```

### 7. Run all test cases at once

Use the standard Cargo test runner to execute all tests in the crate:

```bash
cargo test
```

This runs unit tests, integration tests, and all test files under `tests/`.

## Production Setup

### 1. Configure production services

Provision production-grade services for:

- PostgreSQL
- Redis

Then set the following environment variables appropriately:

```bash
export DATABASE_URL="postgres://<user>:<password>@<host>:5432/task_api"
export REDIS_URL="redis://<host>:6379"
export RUST_LOG=info
```

### 2. Set a secure JWT secret

This repository currently uses a hardcoded JWT secret in `src/auth.rs`:

```rust
const JWT_SECRET: &str = "your-secret-key-change-in-production";
```

Before production submission, replace that string with a strong secret or extend the code to load a secret from an environment variable.

### 3. Run migrations against production database

```bash
sqlx migrate run --database-url "$DATABASE_URL"
```

### 4. Build a release binary

```bash
cargo build --release
```

### 5. Run the server in production mode

```bash
DATABASE_URL="$DATABASE_URL" REDIS_URL="$REDIS_URL" RUST_LOG=info ./target/release/task_api
```

### 6. Optional Docker deployment

If you prefer Docker, build a container image and run with the same environment variables.

Example Dockerfile workflow (not included in repo):

```Dockerfile
FROM rust:1.70-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /app/target/release/task_api /usr/local/bin/task_api
ENV RUST_LOG=info
CMD ["/usr/local/bin/task_api"]
```

Then run:

```bash
docker build -t task_api:latest .
docker run -e DATABASE_URL="$DATABASE_URL" -e REDIS_URL="$REDIS_URL" -e RUST_LOG=info -p 3000:3000 task_api:latest
```

## Submission Checklist

- [ ] Start `postgres` and `redis` services
- [ ] Set `DATABASE_URL` and `REDIS_URL`
- [ ] Run `sqlx migrate run`
- [ ] Build and start the API
- [ ] Run `./validate.sh` or `cargo test --test validation_flow -- --nocapture`
- [ ] Confirm `POST /seed/users`, `POST /auth/login`, `POST /auth/verify-2fa`, `POST /tasks`, `POST /tasks/assign`, and `GET /tasks/view-my-tasks` work
- [ ] Change `JWT_SECRET` to a secure production value

## Notes

- Redis caching is used for `GET /tasks/view-my-tasks`, with cache invalidation on task assignment.
- The `/dev/email-logs/latest` endpoint is only for local development and testing.
- The `/seed/users` endpoint seeds one admin and one staff user: `admin@example.com` / `admin123` and `jamesbond@example.com` / `james123`.
