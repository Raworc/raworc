# Agent Management Testing Plan

## Prerequisites
1. PostgreSQL database running
2. DATABASE_URL environment variable set
3. Run migrations:
   ```bash
   psql $DATABASE_URL < migrations/001_create_rbac_tables.sql
   psql $DATABASE_URL < migrations/002_create_agents_table.sql
   ```

## Test Cases

### 1. List Agents
```bash
# Start server
raworc start

# In another terminal, authenticate
raworc auth

# List agents (should show default "assistant" agent)
raworc connect
raworc> /api agents
```

### 2. Create Agent
```bash
# Create a new agent
raworc> /api POST agents {"name":"code-helper","instructions":"Help with coding tasks","model":"gpt-4","description":"Coding assistant"}

# Verify creation
raworc> /api agents/code-helper
```

### 3. Update Agent
```bash
# Update agent instructions
raworc> /api PUT agents/code-helper {"instructions":"Expert coding assistant focused on best practices"}

# Update multiple fields
raworc> /api PUT agents/code-helper {"model":"gpt-4-turbo","tools":["linter","formatter"],"active":true}
```

### 4. Agent with Complex Fields
```bash
# Create agent with all fields
raworc> /api POST agents {
  "name": "security-reviewer",
  "description": "Security-focused code reviewer",
  "instructions": "Review code for security vulnerabilities",
  "model": "gpt-4",
  "tools": ["semgrep", "bandit", "safety"],
  "routes": [
    {"pattern": "*.py", "priority": 1},
    {"pattern": "*.js", "priority": 2}
  ],
  "guardrails": ["no-eval", "no-exec", "input-validation"],
  "knowledge_bases": ["owasp-top-10", "cwe-database"]
}
```

### 5. Delete Agent
```bash
# Soft delete an agent
raworc> /api DELETE agents/code-helper

# Verify it's no longer in active list
raworc> /api agents

# Try to get deleted agent (should return 404)
raworc> /api agents/code-helper
```

### 6. Error Cases
```bash
# Duplicate name
raworc> /api POST agents {"name":"assistant","instructions":"test","model":"gpt-4"}
# Expected: 409 Conflict

# Invalid ID format
raworc> /api PUT agents/invalid-uuid {"model":"gpt-4"}
# Expected: 400 Bad Request

# Non-existent agent
raworc> /api agents/non-existent
# Expected: 404 Not Found
```

## Swagger UI Testing
1. Navigate to http://localhost:9000/swagger-ui/
2. Use the "Authorize" button to add Bearer token
3. Test all agent endpoints through the UI

## Expected Behaviors
- Agents are soft-deleted (active=false) rather than removed
- Agent names must be unique
- JSON fields (tools, routes, guardrails, knowledge_bases) default to empty arrays
- Timestamps are automatically managed
- Can lookup agents by either UUID or name