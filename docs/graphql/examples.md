# GraphQL Examples

## Authentication

### Service Account Login
```graphql
mutation {
  generateServiceToken(input: {
    user: "admin"
    pass: "admin"
  }) {
    token
    expires_at
  }
}
```

### External Subject Token
```graphql
mutation {
  generateExternalToken(input: {
    subject: "user123"
  }) {
    token
    expires_at
  }
}
```

## Service Accounts

### Create Service Account
```graphql
mutation {
  createServiceAccount(input: {
    user: "myapp"
    pass: "secret123"
    description: "Application service account"
  }) {
    id
    user
    active
  }
}
```

### Delete Service Account
```graphql
mutation {
  deleteServiceAccount(user: "myapp") 
}
```

### List Service Accounts
```graphql
query {
  serviceAccounts {
    user
    namespace
    active
    created_at
  }
}
```

## Roles

### Create Role
```graphql
mutation {
  createRole(input: {
    name: "developer"
    rules: [{
      api_groups: ["*"]
      resources: ["serviceaccounts", "roles"]
      verbs: ["get", "list"]
    }]
  }) {
    name
    rules {
      verbs
      resources
    }
  }
}
```

### List Roles
```graphql
query {
  roles {
    name
    namespace
    description
  }
}
```

## Role Bindings

### Create Role Binding
```graphql
mutation {
  createRoleBinding(input: {
    name: "dev-binding"
    role_ref: {
      kind: "Role"
      name: "developer"
      api_group: "rbac"
    }
    subjects: [{
      kind: ServiceAccount
      name: "myapp"
    }]
  }) {
    name
    role_ref {
      name
    }
  }
}
```

### List Role Bindings
```graphql
query {
  roleBindings {
    name
    role_ref {
      name
    }
    subjects {
      name
      kind
    }
  }
}
```

## Utility

### Check Current User
```graphql
query {
  whoami
}
```

### Server Status
```graphql
query {
  version
  status
}
```