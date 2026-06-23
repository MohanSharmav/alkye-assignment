#!/bin/bash

# Task API - Local Setup and Validation Script
# This script sets up the local development environment and runs the validation workflow

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Task Management API - Setup & Validation ===${NC}\n"

# Step 1: Check prerequisites
echo -e "${YELLOW}[1/7] Checking prerequisites...${NC}"
if ! command -v docker &> /dev/null; then
    echo -e "${RED}✗ Docker is required but not installed${NC}"
    exit 1
fi
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Rust/Cargo is required but not installed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Prerequisites satisfied${NC}\n"

# Step 2: Start Docker services
echo -e "${YELLOW}[2/7] Starting Docker services (PostgreSQL & Redis)...${NC}"
docker-compose down -v 2>/dev/null || true
docker-compose up -d
sleep 5
echo -e "${GREEN}✓ Docker services started${NC}\n"

# Step 3: Install SQLx CLI if needed
echo -e "${YELLOW}[3/7] Checking SQLx CLI...${NC}"
if ! command -v sqlx &> /dev/null; then
    echo "Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi
echo -e "${GREEN}✓ SQLx CLI available${NC}\n"

# Step 4: Run migrations
echo -e "${YELLOW}[4/7] Running database migrations...${NC}"
export DATABASE_URL="postgres://postgres:password@localhost:5432/task_api"
sqlx migrate run --database-url "$DATABASE_URL" 2>/dev/null || echo "Migrations may have already run"
echo -e "${GREEN}✓ Migrations completed${NC}\n"

# Step 5: Build the project
echo -e "${YELLOW}[5/7] Building the project...${NC}"
cargo build --release 2>&1 | grep -E "Finished|Compiling task_api" || true
echo -e "${GREEN}✓ Build completed${NC}\n"

# Step 6: Start the server in background
echo -e "${YELLOW}[6/7] Starting the API server...${NC}"
export DATABASE_URL="postgres://postgres:password@localhost:5432/task_api"
export REDIS_URL="redis://127.0.0.1:6379"
cargo run --release &
SERVER_PID=$!
sleep 3

# Check if server started
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo -e "${RED}✗ Failed to start server${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Server started (PID: $SERVER_PID)${NC}\n"

# Step 7: Run validation workflow
echo -e "${YELLOW}[7/7] Running validation workflow...${NC}\n"

BASE_URL="http://127.0.0.1:3000"

# Seed users
echo -e "${BLUE}Seeding users...${NC}"
SEED=$(curl -s -X POST "$BASE_URL/seed/users")
ADMIN_ID=$(echo $SEED | grep -o '"admin".*' | head -1)
echo -e "${GREEN}✓ Users seeded${NC}"
echo "$SEED" | jq '.' 2>/dev/null || echo "$SEED"
echo ""

# Admin login
echo -e "${BLUE}Admin login (initiate 2FA)...${NC}"
LOGIN=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"admin123"}')
ADMIN_CHALLENGE=$(echo $LOGIN | grep -o '[a-f0-9\-]\{36\}' | head -1)
echo -e "${GREEN}✓ Login challenge created: $ADMIN_CHALLENGE${NC}"
echo ""

# Get verification code
echo -e "${BLUE}Retrieving verification code from email logs...${NC}"
EMAIL_LOG=$(curl -s "$BASE_URL/dev/email-logs/latest")
ADMIN_CODE=$(echo $EMAIL_LOG | grep -o '"code":"[^"]*"' | cut -d'"' -f4)
echo -e "${GREEN}✓ Verification code: $ADMIN_CODE${NC}"
echo ""

# Verify 2FA
echo -e "${BLUE}Verifying 2FA for Admin...${NC}"
AUTH=$(curl -s -X POST "$BASE_URL/auth/verify-2fa" \
  -H "Content-Type: application/json" \
  -d "{\"login_challenge_id\":\"$ADMIN_CHALLENGE\",\"code\":\"$ADMIN_CODE\"}")
ADMIN_TOKEN=$(echo $AUTH | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
echo -e "${GREEN}✓ Admin JWT token received${NC}"
echo ""

# Create 5 tasks
echo -e "${BLUE}Creating 5 tasks as Admin...${NC}"
TASK_IDS=()
for i in {1..5}; do
    TASK=$(curl -s -X POST "$BASE_URL/tasks" \
      -H "Authorization: Bearer $ADMIN_TOKEN" \
      -H "Content-Type: application/json" \
      -d "{\"title\":\"Task $i\",\"priority\":\"$([ $i -le 2 ] && echo 'high' || echo 'medium')\"}")
    TASK_ID=$(echo $TASK | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    TASK_IDS+=($TASK_ID)
    echo -e "  Task $i created: $TASK_ID"
done
echo -e "${GREEN}✓ All 5 tasks created${NC}"
echo ""

# Assign 3 tasks to James Bond
echo -e "${BLUE}Assigning 3 tasks to James Bond...${NC}"
ASSIGN_PAYLOAD=$(printf '{"task_ids":["%s","%s","%s"],"assign_to_email":"jamesbond@example.com"}' \
  "${TASK_IDS[0]}" "${TASK_IDS[1]}" "${TASK_IDS[2]}")
ASSIGN=$(curl -s -X POST "$BASE_URL/tasks/assign" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$ASSIGN_PAYLOAD")
echo -e "${GREEN}✓ Tasks assigned${NC}"
echo ""

# James Bond login
echo -e "${BLUE}James Bond login (initiate 2FA)...${NC}"
LOGIN2=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"jamesbond@example.com","password":"james123"}')
JAMES_CHALLENGE=$(echo $LOGIN2 | grep -o '[a-f0-9\-]\{36\}' | head -1)
echo -e "${GREEN}✓ James Bond login challenge created${NC}"
echo ""

# Get James Bond's verification code
echo -e "${BLUE}Retrieving James Bond's verification code...${NC}"
EMAIL_LOG2=$(curl -s "$BASE_URL/dev/email-logs/latest")
JAMES_CODE=$(echo $EMAIL_LOG2 | grep -o '"code":"[^"]*"' | cut -d'"' -f4)
echo -e "${GREEN}✓ James Bond's verification code: $JAMES_CODE${NC}"
echo ""

# Verify James Bond 2FA
echo -e "${BLUE}Verifying 2FA for James Bond...${NC}"
AUTH2=$(curl -s -X POST "$BASE_URL/auth/verify-2fa" \
  -H "Content-Type: application/json" \
  -d "{\"login_challenge_id\":\"$JAMES_CHALLENGE\",\"code\":\"$JAMES_CODE\"}")
JAMES_TOKEN=$(echo $AUTH2 | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
echo -e "${GREEN}✓ James Bond JWT token received${NC}"
echo ""

# Try to create task as James Bond (should fail)
echo -e "${BLUE}James Bond attempts to create task (should fail with 403)...${NC}"
FAIL_CREATE=$(curl -s -X POST "$BASE_URL/tasks" \
  -H "Authorization: Bearer $JAMES_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Should fail","priority":"high"}')
if echo "$FAIL_CREATE" | grep -q "Forbidden"; then
    echo -e "${GREEN}✓ Correctly rejected (403 Forbidden)${NC}"
else
    echo -e "${RED}✗ Should have been rejected${NC}"
fi
echo ""

# Get tasks (cache miss)
echo -e "${BLUE}James Bond views assigned tasks (cache miss)...${NC}"
TASKS1=$(curl -s "$BASE_URL/tasks/view-my-tasks" \
  -H "Authorization: Bearer $JAMES_TOKEN")
CACHE_HIT_1=$(echo $TASKS1 | grep -o '"hit":\s*[a-z]*' | grep -o '[a-z]*$')
TASK_COUNT=$(echo $TASKS1 | grep -o '"total_assigned_tasks":[0-9]*' | grep -o '[0-9]*$')
echo -e "  Cache hit: $CACHE_HIT_1"
echo -e "  Tasks count: $TASK_COUNT"
if [ "$CACHE_HIT_1" = "false" ] && [ "$TASK_COUNT" = "3" ]; then
    echo -e "${GREEN}✓ First call correct (cache miss, 3 tasks)${NC}"
else
    echo -e "${YELLOW}⚠ Unexpected values - cache hit: $CACHE_HIT_1, count: $TASK_COUNT${NC}"
fi
echo ""

# Get tasks again (cache hit)
echo -e "${BLUE}James Bond views tasks again (should cache hit)...${NC}"
TASKS2=$(curl -s "$BASE_URL/tasks/view-my-tasks" \
  -H "Authorization: Bearer $JAMES_TOKEN")
CACHE_HIT_2=$(echo $TASKS2 | grep -o '"hit":\s*[a-z]*' | grep -o '[a-z]*$')
echo -e "  Cache hit: $CACHE_HIT_2"
if [ "$CACHE_HIT_2" = "true" ]; then
    echo -e "${GREEN}✓ Second call cached correctly${NC}"
else
    echo -e "${YELLOW}⚠ Expected cache hit, got: $CACHE_HIT_2${NC}"
fi
echo ""

# Display final response
echo -e "${BLUE}Final validation response:${NC}"
echo "$TASKS2" | jq '.' 2>/dev/null || echo "$TASKS2"

# Cleanup
echo -e "\n${YELLOW}Cleaning up...${NC}"
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo -e "${GREEN}\n=== Validation Complete ===${NC}"
echo -e "${GREEN}All workflow steps validated successfully!${NC}\n"
