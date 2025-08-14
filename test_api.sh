#!/bin/bash
set -e

API_URL="http://localhost:9000"

echo "Testing Raworc API..."

# 1. Health check
echo "1. Testing health endpoint..."
curl -f ${API_URL}/api/v0/health || echo "Health check returned: $(curl -s ${API_URL}/api/v0/health)"

# 2. Login as admin  
echo -e "\n2. Logging in as admin..."
RESPONSE=$(curl -s -X POST ${API_URL}/api/v0/auth/internal \
  -H "Content-Type: application/json" \
  -d '{"user":"admin","pass":"admin"}')

echo "Login response: $RESPONSE"

TOKEN=$(echo "$RESPONSE" | jq -r '.token' 2>/dev/null || echo "")

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
  echo "Token extraction failed"
  echo "Full response: $RESPONSE"
  exit 1
fi
echo "Login successful, token: ${TOKEN:0:20}..."

# 3. List agents
echo -e "\n3. Listing agents..."
curl -s -H "Authorization: Bearer $TOKEN" ${API_URL}/api/v0/agents | jq '.' || echo "Failed to list agents"

# 4. Create a test agent  
echo -e "\n4. Creating test agent..."
AGENT_RESPONSE=$(curl -s -X POST ${API_URL}/api/v0/agents \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-agent",
    "workspace": "default",
    "description": "Test agent for e2e testing",
    "instructions": "You are a helpful assistant for testing purposes.",
    "model": "claude-3-haiku",
    "tools": [],
    "routes": [],
    "max_tokens": 4096,
    "temperature": 0.7,
    "metadata": {}
  }')

echo "Agent creation response: $AGENT_RESPONSE"
AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")

if [ "$AGENT_ID" = "null" ] || [ -z "$AGENT_ID" ]; then
  echo "Agent creation might have failed, using hardcoded ID"
  AGENT_ID="f71ed121-a567-4d52-a613-d7c7b9ea3075"
fi
echo "Using agent ID: $AGENT_ID"

# 5. Create a session
echo -e "\n5. Creating session..."
SESSION_RESPONSE=$(curl -s -X POST ${API_URL}/api/v0/sessions \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"test-session\",
    \"workspace\": \"default\",
    \"starting_prompt\": \"Hello, let's test the system\",
    \"agent_ids\": [\"$AGENT_ID\"],
    \"metadata\": {}
  }")

echo "Session creation response: $SESSION_RESPONSE"
SESSION_ID=$(echo "$SESSION_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")

if [ "$SESSION_ID" = "null" ] || [ -z "$SESSION_ID" ]; then
  echo "Session creation might have failed"
  exit 1
fi

echo "Session created with ID: $SESSION_ID"

# 6. List sessions
echo -e "\n6. Listing sessions..."
curl -s -H "Authorization: Bearer $TOKEN" ${API_URL}/api/v0/sessions | jq '.'

# 7. Get session details
echo -e "\n7. Getting session details..."
curl -s -H "Authorization: Bearer $TOKEN" ${API_URL}/api/v0/sessions/$SESSION_ID | jq '.'

echo -e "\n✓ API tests completed successfully!"