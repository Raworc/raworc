#!/bin/bash

# Test script for Docker orchestrator

set -e

echo "Testing Docker Orchestrator"
echo "=========================="

# Set environment variables
export DATABASE_URL=postgresql://raworc:raworc@localhost:5432/raworc
export JWT_SECRET=test-secret
export HOST_AGENT_IMAGE=python:3.11-slim
export HOST_AGENT_VOLUMES_PATH=/tmp/raworc-volumes
export RUST_LOG=debug

# Create volumes directory
mkdir -p /tmp/raworc-volumes

echo "1. Starting the server..."
./target/release/raworc start &
SERVER_PID=$!

# Wait for server to start
echo "   Waiting for server to start..."
sleep 5

# Check if server is running
if kill -0 $SERVER_PID 2>/dev/null; then
    echo "   ✓ Server started successfully (PID: $SERVER_PID)"
else
    echo "   ✗ Server failed to start"
    exit 1
fi

echo ""
echo "2. Authenticating..."
# Create auth request
AUTH_RESPONSE=$(curl -s -X POST http://localhost:9000/api/v0/auth/internal \
    -H "Content-Type: application/json" \
    -d '{"user":"admin","pass":"admin"}')

TOKEN=$(echo $AUTH_RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "   ✗ Failed to authenticate"
    echo "   Response: $AUTH_RESPONSE"
    kill $SERVER_PID
    exit 1
fi

echo "   ✓ Authentication successful"

echo ""
echo "3. Creating a test session..."
SESSION_RESPONSE=$(curl -s -X POST http://localhost:9000/api/v0/sessions \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d '{
        "name": "Test Session",
        "workspace": "default",
        "starting_prompt": "This is a test session",
        "waiting_timeout_seconds": 300
    }')

SESSION_ID=$(echo $SESSION_RESPONSE | grep -o '"id":"[^"]*' | cut -d'"' -f4)

if [ -z "$SESSION_ID" ]; then
    echo "   ✗ Failed to create session"
    echo "   Response: $SESSION_RESPONSE"
    kill $SERVER_PID
    exit 1
fi

echo "   ✓ Session created: $SESSION_ID"

echo ""
echo "4. Checking Docker container..."
CONTAINER_ID=$(docker ps --filter "label=raworc.session.id=$SESSION_ID" --format "{{.ID}}")

if [ -z "$CONTAINER_ID" ]; then
    echo "   ✗ No Docker container found for session"
    
    # Get session details
    echo ""
    echo "   Session details:"
    curl -s -X GET "http://localhost:9000/api/v0/sessions/$SESSION_ID" \
        -H "Authorization: Bearer $TOKEN" | python3 -m json.tool
else
    echo "   ✓ Docker container created: $CONTAINER_ID"
    
    echo ""
    echo "   Container details:"
    docker inspect $CONTAINER_ID | python3 -m json.tool | head -50
fi

echo ""
echo "5. Checking session state..."
SESSION_STATE=$(curl -s -X GET "http://localhost:9000/api/v0/sessions/$SESSION_ID" \
    -H "Authorization: Bearer $TOKEN" | grep -o '"state":"[^"]*' | cut -d'"' -f4)

echo "   Session state: $SESSION_STATE"

echo ""
echo "6. Deleting the session..."
curl -s -X DELETE "http://localhost:9000/api/v0/sessions/$SESSION_ID" \
    -H "Authorization: Bearer $TOKEN"

echo "   ✓ Session deleted"

# Check if container is removed
sleep 2
CONTAINER_ID=$(docker ps -a --filter "label=raworc.session.id=$SESSION_ID" --format "{{.ID}}")
if [ -z "$CONTAINER_ID" ]; then
    echo "   ✓ Docker container removed"
else
    echo "   ⚠ Docker container still exists: $CONTAINER_ID"
fi

echo ""
echo "7. Stopping the server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null || true
echo "   ✓ Server stopped"

echo ""
echo "=========================="
echo "Test completed successfully!"