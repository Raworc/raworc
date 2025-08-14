#!/bin/bash
set -e

echo "Testing README commands..."

# 1. Get auth token
echo "1. Getting auth token..."
TOKEN=$(curl -s -X POST http://localhost:9000/api/v0/auth/internal \
  -H "Content-Type: application/json" \
  -d '{"user":"admin","pass":"admin"}' | jq -r '.token')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
  echo "Failed to get token"
  exit 1
fi
echo "✓ Got token: ${TOKEN:0:20}..."

# 2. Create an agent
echo -e "\n2. Creating agent..."
AGENT_RESPONSE=$(curl -s -X POST http://localhost:9000/api/v0/agents \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-agent",
    "workspace": "default", 
    "instructions": "You are a helpful assistant",
    "model": "claude-3-haiku",
    "tools": [],
    "routes": []
  }')

echo "Agent response: $AGENT_RESPONSE"
AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")

if [ -z "$AGENT_ID" ] || [ "$AGENT_ID" = "null" ]; then
  echo "Note: Agent creation returned an error (may already exist)"
  # Try to get existing agent
  AGENT_ID=$(curl -s -H "Authorization: Bearer $TOKEN" http://localhost:9000/api/v0/agents | jq -r '.[0].id' 2>/dev/null || echo "")
  if [ -n "$AGENT_ID" ] && [ "$AGENT_ID" != "null" ]; then
    echo "✓ Using existing agent: $AGENT_ID"
  else
    echo "Failed to get agent ID"
    exit 1
  fi
else
  echo "✓ Created agent: $AGENT_ID"
fi

# 3. Create a session
echo -e "\n3. Creating session..."
SESSION_RESPONSE=$(curl -s -X POST http://localhost:9000/api/v0/sessions \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"test-session\",
    \"workspace\": \"default\",
    \"starting_prompt\": \"Hello\",
    \"agent_ids\": [\"$AGENT_ID\"]
  }")

SESSION_ID=$(echo "$SESSION_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")

if [ -z "$SESSION_ID" ] || [ "$SESSION_ID" = "null" ]; then
  echo "Failed to create session"
  echo "Response: $SESSION_RESPONSE"
  exit 1
fi
echo "✓ Created session: $SESSION_ID"

# 4. Check session status
echo -e "\n4. Checking session status..."
SESSION_STATUS=$(curl -s -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/api/v0/sessions/$SESSION_ID | jq -r '.state' 2>/dev/null || echo "")

if [ -n "$SESSION_STATUS" ] && [ "$SESSION_STATUS" != "null" ]; then
  echo "✓ Session status: $SESSION_STATUS"
else
  echo "Failed to get session status"
  exit 1
fi

echo -e "\n✅ All README commands work!"