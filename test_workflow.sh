#!/bin/bash
set -e

BASE_URL="http://127.0.0.1:3000"

echo "========================================="
echo "1. Seeding Users"
echo "========================================="
curl -s -X POST "$BASE_URL/seed/users" | jq .

echo ""
echo "========================================="
echo "2. Admin Login (Trigger 2FA)"
echo "========================================="
LOGIN_RES=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@example.com", "password": "admin123"}')

echo $LOGIN_RES | jq .
CHALLENGE_ID=$(echo $LOGIN_RES | jq -r .login_challenge_id)

echo ""
echo "========================================="
echo "3. Fetch Admin 2FA Code from Email Logs"
echo "========================================="
EMAIL_LOG=$(curl -s -X GET "$BASE_URL/dev/email-logs/latest")
echo $EMAIL_LOG | jq .
ADMIN_CODE=$(echo $EMAIL_LOG | jq -r .code)

echo ""
echo "========================================="
echo "4. Verify Admin 2FA and get JWT"
echo "========================================="
VERIFY_RES=$(curl -s -X POST "$BASE_URL/auth/verify-2fa" \
  -H "Content-Type: application/json" \
  -d "{\"login_challenge_id\": \"$CHALLENGE_ID\", \"code\": \"$ADMIN_CODE\"}")
echo $VERIFY_RES | jq .
ADMIN_TOKEN=$(echo $VERIFY_RES | jq -r .access_token)

echo ""
echo "========================================="
echo "5. Create 5 Tasks as Admin"
echo "========================================="
TASK_IDS=()
for i in {1..5}; do
  TASK_RES=$(curl -s -X POST "$BASE_URL/tasks" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -d "{\"title\": \"Task $i\", \"description\": \"Description for task $i\", \"priority\": \"high\"}")
  
  TASK_ID=$(echo $TASK_RES | jq -r .id)
  echo "Created Task ID: $TASK_ID"
  TASK_IDS+=("$TASK_ID")
done

echo ""
echo "========================================="
echo "6. Assign 3 Tasks to James Bond"
echo "========================================="
ASSIGN_RES=$(curl -s -X POST "$BASE_URL/tasks/assign" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d "{
    \"task_ids\": [\"${TASK_IDS[0]}\", \"${TASK_IDS[1]}\", \"${TASK_IDS[2]}\"],
    \"assign_to_email\": \"jamesbond@example.com\"
  }")
echo $ASSIGN_RES | jq .

echo ""
echo "========================================="
echo "7. James Bond Login (Trigger 2FA)"
echo "========================================="
JB_LOGIN_RES=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email": "jamesbond@example.com", "password": "james123"}')
echo $JB_LOGIN_RES | jq .
JB_CHALLENGE_ID=$(echo $JB_LOGIN_RES | jq -r .login_challenge_id)

echo ""
echo "========================================="
echo "8. Fetch James Bond 2FA Code from Email Logs"
echo "========================================="
JB_EMAIL_LOG=$(curl -s -X GET "$BASE_URL/dev/email-logs/latest")
echo $JB_EMAIL_LOG | jq .
JB_CODE=$(echo $JB_EMAIL_LOG | jq -r .code)

echo ""
echo "========================================="
echo "9. Verify James Bond 2FA and get JWT"
echo "========================================="
JB_VERIFY_RES=$(curl -s -X POST "$BASE_URL/auth/verify-2fa" \
  -H "Content-Type: application/json" \
  -d "{\"login_challenge_id\": \"$JB_CHALLENGE_ID\", \"code\": \"$JB_CODE\"}")
echo $JB_VERIFY_RES | jq .
JB_TOKEN=$(echo $JB_VERIFY_RES | jq -r .access_token)

echo ""
echo "========================================="
echo "10. Attempt Task Creation as James Bond (Should fail with 403)"
echo "========================================="
curl -s -w "\nHTTP Status: %{http_code}\n" -X POST "$BASE_URL/tasks" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JB_TOKEN" \
  -d '{"title": "JB Secret Mission", "description": "Classified", "priority": "high"}'

echo ""
echo "========================================="
echo "11. View Tasks as James Bond (1st time: cache miss)"
echo "========================================="
curl -s -X GET "$BASE_URL/tasks/view-my-tasks" \
  -H "Authorization: Bearer $JB_TOKEN" | jq .

echo ""
echo "========================================="
echo "12. View Tasks as James Bond (2nd time: cache hit)"
echo "========================================="
curl -s -X GET "$BASE_URL/tasks/view-my-tasks" \
  -H "Authorization: Bearer $JB_TOKEN" | jq .
