#!/bin/bash

# Test Raworc API

API_URL="http://localhost:9000"

echo "=== Testing Raworc API ==="

# 1. Check health
echo -e "\n1. Checking health endpoint..."
curl -s $API_URL/health | jq . || echo "Health check failed"

# 2. Login to get token
echo -e "\n2. Logging in as admin..."
TOKEN=$(curl -s -X POST $API_URL/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"user": "admin", "pass": "admin"}' | jq -r .token)

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
    echo "Login failed. Server may not be running or RBAC not seeded."
    exit 1
fi

echo "Got token: ${TOKEN:0:20}..."

# 3. List sessions
echo -e "\n3. Listing sessions..."
curl -s $API_URL/api/v1/sessions \
  -H "Authorization: Bearer $TOKEN" | jq .

# 4. Create a session
echo -e "\n4. Creating a new session..."
SESSION=$(curl -s -X POST $API_URL/api/v1/sessions \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Session",
    "starting_prompt": "Hello, this is a test session"
  }' | jq .)

echo "$SESSION"
SESSION_ID=$(echo "$SESSION" | jq -r .id)

if [ "$SESSION_ID" = "null" ] || [ -z "$SESSION_ID" ]; then
    echo "Failed to create session"
    exit 1
fi

echo "Created session: $SESSION_ID"

# 5. Get session details
echo -e "\n5. Getting session details..."
curl -s $API_URL/api/v1/sessions/$SESSION_ID \
  -H "Authorization: Bearer $TOKEN" | jq .

# 6. Send a message
echo -e "\n6. Sending a message to session..."
MESSAGE=$(curl -s -X POST $API_URL/api/v1/sessions/$SESSION_ID/messages \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "USER",
    "content": "Hello from test script!"
  }' | jq .)

echo "$MESSAGE"

# 7. List messages
echo -e "\n7. Listing session messages..."
curl -s $API_URL/api/v1/sessions/$SESSION_ID/messages \
  -H "Authorization: Bearer $TOKEN" | jq .

# 8. Delete session
echo -e "\n8. Deleting session..."
curl -s -X DELETE $API_URL/api/v1/sessions/$SESSION_ID \
  -H "Authorization: Bearer $TOKEN"

echo -e "\n=== API Test Complete ==="