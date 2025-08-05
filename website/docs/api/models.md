---
sidebar_position: 3
title: Data Models
---

# Data Models

Complete reference for all data models used in the Raworc API.

## Agent Models

### Agent

Represents an AI agent configuration.

```typescript
interface Agent {
  id: string;                    // UUID
  name: string;                  // Unique agent name
  description?: string;          // Optional description
  instructions: string;          // System instructions for the agent
  model: string;                 // AI model to use (e.g., "gpt-4")
  tools: any[];                  // Array of tool configurations
  routes: any[];                 // Routing rules for the agent
  guardrails: any[];            // Safety and constraint rules
  knowledge_bases: any[];       // Knowledge base references
  active: boolean;              // Whether agent is active
  created_at: string;           // ISO 8601 timestamp
  updated_at: string;           // ISO 8601 timestamp
}
```

### CreateAgentRequest

Request body for creating a new agent.

```typescript
interface CreateAgentRequest {
  name: string;                  // Required, must be unique
  description?: string;          // Optional
  instructions: string;          // Required
  model: string;                 // Required (e.g., "gpt-4", "gpt-3.5-turbo")
  tools?: any[];                 // Optional, defaults to []
  routes?: any[];                // Optional, defaults to []
  guardrails?: any[];           // Optional, defaults to []
  knowledge_bases?: any[];      // Optional, defaults to []
}
```

### UpdateAgentRequest

Request body for updating an agent (all fields optional).

```typescript
interface UpdateAgentRequest {
  name?: string;
  description?: string;
  instructions?: string;
  model?: string;
  tools?: any[];
  routes?: any[];
  guardrails?: any[];
  knowledge_bases?: any[];
  active?: boolean;
}
```

## Session Models

### Session

Represents a work session with assigned agents.

```typescript
interface Session {
  id: string;                           // UUID
  name: string;                         // Session name
  starting_prompt: string;              // Initial prompt/context
  lifecycle_state: SessionLifecycle;    // Current state
  waiting_timeout_seconds?: number;     // Auto-terminate timeout
  container_id?: string;                // Kubernetes container ID
  persistent_volume_id?: string;        // PV for state storage
  created_by: string;                   // Username who created
  parent_session_id?: string;           // UUID of parent (for remixed)
  agents: SessionAgentInfo[];           // Assigned agents
  created_at: string;                   // ISO 8601 timestamp
  started_at?: string;                  // When session started
  last_activity_at?: string;            // Last activity time
  terminated_at?: string;               // When terminated
  termination_reason?: string;          // Why terminated
  metadata: object;                     // Custom metadata
}
```

### SessionLifecycle

Enum representing session states.

```typescript
enum SessionLifecycle {
  NOT_STARTED = "NOT_STARTED",   // Initial state
  STARTED = "STARTED",           // Ready for work
  BUSY = "BUSY",                 // Processing work
  WAITING = "WAITING",           // Idle, waiting
  TERMINATED = "TERMINATED"      // Ended
}
```

### SessionAgentInfo

Simplified agent information in session responses.

```typescript
interface SessionAgentInfo {
  id: string;        // Agent UUID
  name: string;      // Agent name
  model: string;     // AI model
}
```

### CreateSessionRequest

Request body for creating a new session.

```typescript
interface CreateSessionRequest {
  name: string;                         // Required
  starting_prompt: string;              // Required
  agent_ids: string[];                  // Array of agent UUIDs
  waiting_timeout_seconds?: number;     // Optional, defaults to 300
  metadata?: object;                    // Optional custom data
}
```

### UpdateSessionRequest

Request body for updating session details.

```typescript
interface UpdateSessionRequest {
  name?: string;
  waiting_timeout_seconds?: number;
  metadata?: object;
}
```

### UpdateSessionStateRequest

Request body for updating session lifecycle state.

```typescript
interface UpdateSessionStateRequest {
  lifecycle_state: SessionLifecycle;    // Required
  container_id?: string;                // For STARTED state
  persistent_volume_id?: string;        // For STARTED state
  termination_reason?: string;          // For TERMINATED state
}
```

### RemixSessionRequest

Request body for creating a remixed session.

```typescript
interface RemixSessionRequest {
  name: string;                         // Required
  starting_prompt?: string;             // Optional, inherits from parent
  agent_ids?: string[];                 // Optional, inherits from parent
  waiting_timeout_seconds?: number;     // Optional, inherits from parent
  metadata?: object;                    // Optional, inherits from parent
}
```

## RBAC Models

### ServiceAccount

Represents a service account for authentication.

```typescript
interface ServiceAccount {
  id: string;              // UUID
  user: string;            // Username (unique)
  pass: string;            // Hashed password
  namespace?: string;      // Optional namespace
  created_at: string;      // ISO 8601 timestamp
  updated_at: string;      // ISO 8601 timestamp
}
```

### Role

Represents a role with permissions.

```typescript
interface Role {
  id: string;              // UUID
  name: string;            // Role name (unique)
  rules: Rule[];           // Permission rules
  created_at: string;      // ISO 8601 timestamp
  updated_at: string;      // ISO 8601 timestamp
}
```

### Rule

Permission rule within a role.

```typescript
interface Rule {
  api_groups: string[];    // API groups (e.g., ["api"])
  resources: string[];     // Resources (e.g., ["agents", "sessions"])
  verbs: string[];        // Actions (e.g., ["get", "list", "create"])
}
```

### RoleBinding

Binds roles to subjects (users/service accounts).

```typescript
interface RoleBinding {
  id: string;              // UUID
  name: string;            // Binding name (unique)
  role_ref: RoleRef;       // Reference to role
  subjects: Subject[];     // Who gets the role
  created_at: string;      // ISO 8601 timestamp
  updated_at: string;      // ISO 8601 timestamp
}
```

### RoleRef

Reference to a role.

```typescript
interface RoleRef {
  api_group: string;       // Usually "rbac"
  kind: string;           // Usually "Role"
  name: string;           // Role name
}
```

### Subject

Subject that receives role permissions.

```typescript
interface Subject {
  kind: SubjectType;       // Type of subject
  name: string;           // Subject name
  namespace?: string;      // Optional namespace
}
```

### SubjectType

Enum for subject types.

```typescript
enum SubjectType {
  User = "User",
  ServiceAccount = "ServiceAccount",
  Group = "Group"
}
```

## Common Types

### Error Response

Standard error response format.

```typescript
interface ErrorResponse {
  error: {
    code: string;         // Error code (e.g., "NOT_FOUND")
    message: string;      // Human-readable message
  };
}
```

### Pagination (Future)

Reserved for future pagination support.

```typescript
interface PaginatedResponse<T> {
  items: T[];             // Array of items
  total: number;          // Total count
  page: number;           // Current page
  per_page: number;       // Items per page
  has_more: boolean;      // More pages available
}
```

## Field Constraints

### Common Constraints

- **UUIDs**: All IDs are v4 UUIDs
- **Timestamps**: ISO 8601 format (e.g., "2025-01-20T10:00:00Z")
- **Names**: Must be unique within their type, alphanumeric with hyphens/underscores

### Agent Constraints

- `name`: 1-255 characters, unique
- `model`: Must be a supported model (e.g., "gpt-4", "gpt-3.5-turbo", "claude-3")
- `instructions`: 1-65535 characters
- `description`: 0-1000 characters

### Session Constraints

- `name`: 1-255 characters
- `starting_prompt`: 1-65535 characters
- `waiting_timeout_seconds`: 0-86400 (24 hours max)
- `metadata`: Valid JSON object, max 65KB

### RBAC Constraints

- `user`: 1-255 characters, unique, alphanumeric
- `pass`: Minimum 8 characters (before hashing)
- `namespace`: 0-255 characters
- `verbs`: Valid values: "get", "list", "create", "update", "delete", "*"