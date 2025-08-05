#!/bin/bash

# Service Account Improvements Test Script

BASE_URL="http://localhost:9000/api/v0"
ADMIN_USER="admin"
ADMIN_PASS="admin"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
PASSED=0
FAILED=0

print_test() {
    echo -e "\n${YELLOW}[TEST]${NC} $1"
}

print_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED++))
}

print_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED++))
}

# Function to login and get token
get_admin_token() {
    local response=$(curl -s -X POST "$BASE_URL/auth/internal" \
        -H "Content-Type: application/json" \
        -d "{\"user\": \"$ADMIN_USER\", \"pass\": \"$ADMIN_PASS\"}")
    
    echo "$response" | jq -r '.token'
}

# Function to login with specific credentials
login_with_creds() {
    local user=$1
    local pass=$2
    local namespace=$3
    
    local data="{\"user\": \"$user\", \"pass\": \"$pass\""
    if [ -n "$namespace" ]; then
        data="${data%\}}, \"namespace\": \"$namespace\"}"
    else
        data="${data}}"
    fi
    
    curl -s -X POST "$BASE_URL/auth/internal" \
        -H "Content-Type: application/json" \
        -d "$data"
}

# Get admin token for authenticated requests
TOKEN=$(get_admin_token)
if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
    echo -e "${RED}Failed to authenticate as admin. Is the server running?${NC}"
    exit 1
fi

echo -e "${GREEN}Successfully authenticated as admin${NC}"

# Test 1: Create a test service account
print_test "Creating test service account"
CREATE_RESPONSE=$(curl -s -X POST "$BASE_URL/service-accounts" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "user": "test-sa",
        "pass": "TestPass123!",
        "namespace": "test-namespace",
        "description": "Test service account for password change"
    }')

SA_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
if [ -n "$SA_ID" ] && [ "$SA_ID" != "null" ]; then
    print_pass "Created service account with ID: $SA_ID"
else
    print_fail "Failed to create service account: $CREATE_RESPONSE"
    exit 1
fi

# Test 2: Change password with correct current password
print_test "Changing password with correct current password"
CHANGE_RESPONSE=$(curl -s -w "\n%{http_code}" -X PUT "$BASE_URL/service-accounts/$SA_ID/password" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "current_password": "TestPass123!",
        "new_password": "NewTestPass456!"
    }')

HTTP_CODE=$(echo "$CHANGE_RESPONSE" | tail -n 1)
BODY=$(echo "$CHANGE_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" = "204" ] || [ "$HTTP_CODE" = "200" ]; then
    print_pass "Password changed successfully"
else
    print_fail "Failed to change password. HTTP: $HTTP_CODE, Response: $BODY"
fi

# Test 3: Verify login with new password
print_test "Verifying login with new password"
LOGIN_RESPONSE=$(login_with_creds "test-sa" "NewTestPass456!" "test-namespace")
LOGIN_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')

if [ -n "$LOGIN_TOKEN" ] && [ "$LOGIN_TOKEN" != "null" ]; then
    print_pass "Successfully logged in with new password"
else
    print_fail "Failed to login with new password: $LOGIN_RESPONSE"
fi

# Test 4: Try to change password with incorrect current password
print_test "Attempting password change with incorrect current password"
FAIL_RESPONSE=$(curl -s -w "\n%{http_code}" -X PUT "$BASE_URL/service-accounts/$SA_ID/password" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "current_password": "WrongPassword123!",
        "new_password": "AnotherNewPass789!"
    }')

HTTP_CODE=$(echo "$FAIL_RESPONSE" | tail -n 1)
BODY=$(echo "$FAIL_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" = "401" ]; then
    print_pass "Correctly rejected password change with wrong current password"
else
    print_fail "Expected 401 but got HTTP: $HTTP_CODE, Response: $BODY"
fi

# Test 5: Update service account fields
print_test "Updating service account fields"
UPDATE_RESPONSE=$(curl -s -X PUT "$BASE_URL/service-accounts/$SA_ID" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "namespace": "updated-namespace",
        "description": "Updated description",
        "active": true
    }')

UPDATED_NS=$(echo "$UPDATE_RESPONSE" | jq -r '.namespace')
UPDATED_DESC=$(echo "$UPDATE_RESPONSE" | jq -r '.description')
UPDATED_ACTIVE=$(echo "$UPDATE_RESPONSE" | jq -r '.active')

if [ "$UPDATED_NS" = "updated-namespace" ] && [ "$UPDATED_DESC" = "Updated description" ] && [ "$UPDATED_ACTIVE" = "true" ]; then
    print_pass "Service account fields updated successfully"
else
    print_fail "Service account fields not updated correctly: $UPDATE_RESPONSE"
fi

# Test 6: Check last_login_at is populated
print_test "Checking last_login_at field"
# First, login again to trigger last_login_at update
LOGIN_RESPONSE=$(login_with_creds "test-sa" "NewTestPass456!" "updated-namespace")
sleep 1

# Get service account details
SA_DETAILS=$(curl -s -X GET "$BASE_URL/service-accounts/$SA_ID" \
    -H "Authorization: Bearer $TOKEN")

LAST_LOGIN=$(echo "$SA_DETAILS" | jq -r '.last_login_at')
if [ -n "$LAST_LOGIN" ] && [ "$LAST_LOGIN" != "null" ]; then
    print_pass "last_login_at is populated: $LAST_LOGIN"
else
    print_fail "last_login_at is not populated: $SA_DETAILS"
fi

# Test 7: Disable account and verify login fails
print_test "Disabling account and verifying login fails"
DISABLE_RESPONSE=$(curl -s -X PUT "$BASE_URL/service-accounts/$SA_ID" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "active": false
    }')

# Try to login with disabled account
LOGIN_RESPONSE=$(login_with_creds "test-sa" "NewTestPass456!" "updated-namespace")
LOGIN_ERROR=$(echo "$LOGIN_RESPONSE" | jq -r '.error')

if [ -n "$LOGIN_ERROR" ] && [ "$LOGIN_ERROR" != "null" ]; then
    print_pass "Login correctly failed for disabled account"
else
    print_fail "Login should have failed for disabled account: $LOGIN_RESPONSE"
fi

# Test 8: Check OpenAPI/Swagger endpoints
print_test "Checking OpenAPI documentation includes new endpoints"
OPENAPI_RESPONSE=$(curl -s "http://localhost:9000/api-docs/openapi.json")

# Check for password endpoint
if echo "$OPENAPI_RESPONSE" | grep -q "/service-accounts/{id}/password"; then
    print_pass "Password change endpoint found in OpenAPI spec"
else
    print_fail "Password change endpoint not found in OpenAPI spec"
fi

# Check for PUT service-accounts endpoint
if echo "$OPENAPI_RESPONSE" | grep -q "put.*service-accounts/{id}"; then
    print_pass "Service account update endpoint found in OpenAPI spec"
else
    print_fail "Service account update endpoint not found in OpenAPI spec"
fi

# Cleanup: Delete test service account
print_test "Cleaning up test service account"
DELETE_RESPONSE=$(curl -s -w "\n%{http_code}" -X DELETE "$BASE_URL/service-accounts/$SA_ID" \
    -H "Authorization: Bearer $TOKEN")

HTTP_CODE=$(echo "$DELETE_RESPONSE" | tail -n 1)
if [ "$HTTP_CODE" = "204" ] || [ "$HTTP_CODE" = "200" ]; then
    print_pass "Test service account deleted"
else
    print_fail "Failed to delete test service account"
fi

# Summary
echo -e "\n${YELLOW}=== TEST SUMMARY ===${NC}"
echo -e "Tests passed: ${GREEN}$PASSED${NC}"
echo -e "Tests failed: ${RED}$FAILED${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}Some tests failed.${NC}"
    exit 1
fi