# Quick Start Guide

Get the Task Management API running in minutes.

## Prerequisites

- Docker and Docker Compose installed
- Rust 1.70+ installed
- curl or Postman for testing

## Installation (3 steps)

### 1. Start dependencies
```bash
docker-compose up -d
```
This starts PostgreSQL and Redis locally.

### 2. Run migrations
```bash
cargo install sqlx-cli --no-default-features --features postgres

export DATABASE_URL="postgres://postgres:password@localhost:5432/task_api"
sqlx migrate run
```

### 3. Run the server
```bash
export DATABASE_URL="postgres://postgres:password@localhost:5432/task_api"
export REDIS_URL="redis://127.0.0.1:6379"

cargo run
```

Server listens on `http://127.0.0.1:3000`

---

## Automated Validation

Run the complete workflow automatically:

```bash
./validate.sh
```

This script:
1. ✓ Seeds users
2. ✓ Tests 2FA authentication
3. ✓ Creates and assigns tasks
4. ✓ Validates caching
5. ✓ Generates the final validation response

---

## Manual Validation

Or follow the steps manually in [README.md](README.md).

### Quick Example
```bash
# 1. Seed users
curl -X POST http://127.0.0.1:3000/seed/users

# 2. Login
curl -X POST http://127.0.0.1:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"admin123"}'

# 3. Get verification code
curl http://127.0.0.1:3000/dev/email-logs/latest

# 4. View response
curl http://127.0.0.1:3000/health
```

---

## Troubleshooting

**Port 5432 (PostgreSQL) already in use?**
```bash
docker-compose down -v
docker-compose up -d
```

**Can't connect to Redis?**
Redis is optional. Cache will gracefully degrade.

**Build errors?**
```bash
cargo clean
cargo build
```

---

## Project Structure

```
.
├── src/
│   ├── main.rs       # Entry point & routing
│   ├── models.rs     # Data structures
│   ├── handlers.rs   # API endpoints
│   ├── auth.rs       # JWT & password hashing
│   ├── db.rs         # Database queries
│   ├── cache.rs      # Redis caching
│   ├── error.rs      # Error handling
│   └── middleware.rs # Authorization
├── migrations/       # SQL migrations
├── docker-compose.yml
├── Cargo.toml
└── README.md
```

---

## Next Steps

1. See [README.md](README.md) for complete documentation
2. Check [AI_USAGE.md](AI_USAGE.md) for development details
3. Review endpoint specs in the main README
4. Explore the code in `src/` directory

---

## Support

- Full validation workflow: See README.md, Step 1-11
- Error responses: See src/error.rs
- Database schema: See migrations/
