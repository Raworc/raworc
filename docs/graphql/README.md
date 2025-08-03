# Raworc GraphQL API

## Authentication
JWT Bearer token in Authorization header:
```
Authorization: Bearer <your-jwt-token>
```

## Endpoint
```
http://localhost:9000/graphql
```

## Queries

### Service Accounts
```graphql
serviceAccounts: [ServiceAccount!]!
serviceAccount(user: String!, namespace: String): ServiceAccount
```

### Roles
```graphql
roles: [Role!]!
role(name: String!, namespace: String): Role
```

### Role Bindings
```graphql
roleBindings: [RoleBinding!]!
roleBinding(name: String!, namespace: String): RoleBinding
```

### Utility
```graphql
version: String!
status: String!
whoami: String!
```

## Mutations

### Authentication
```graphql
generateServiceToken(input: ServiceAccountLoginInput!): TokenResponse!
generateExternalToken(input: SubjectLoginInput!): TokenResponse!
```

### Service Account Management
```graphql
createServiceAccount(input: CreateServiceAccountInput!): ServiceAccount!
deleteServiceAccount(user: String!, namespace: String): Boolean!
deleteServiceAccountById(id: String!): Boolean!
```

### Role Management
```graphql
createRole(input: CreateRoleInput!): Role!
deleteRole(name: String!, namespace: String): Boolean!
setupAdminRole: Role!
```

### Role Binding Management
```graphql
createRoleBinding(input: CreateRoleBindingInput!): RoleBinding!
deleteRoleBinding(name: String!, namespace: String): Boolean!
```

## Types

### ServiceAccount
```graphql
type ServiceAccount {
  id: ID
  user: String!
  namespace: String
  description: String
  active: Boolean!
  created_at: String!
}
```

### Role
```graphql
type Role {
  id: ID
  name: String!
  namespace: String
  rules: [Rule!]!
  description: String
  created_at: String!
}
```

### Rule
```graphql
type Rule {
  api_groups: [String!]!
  resources: [String!]!
  verbs: [String!]!
  resource_names: [String!]
}
```

### RoleBinding
```graphql
type RoleBinding {
  id: ID
  name: String!
  namespace: String
  role_ref: RoleRef!
  subjects: [RoleBindingSubject!]!
  created_at: String!
}
```

### Input Types

#### ServiceAccountLoginInput
```graphql
input ServiceAccountLoginInput {
  user: String!
  namespace: String
  pass: String!
}
```

#### CreateServiceAccountInput
```graphql
input CreateServiceAccountInput {
  user: String!
  namespace: String
  pass: String!
  description: String
}
```

#### CreateRoleInput
```graphql
input CreateRoleInput {
  name: String!
  namespace: String
  rules: [RuleInput!]!
  description: String
}
```

#### CreateRoleBindingInput
```graphql
input CreateRoleBindingInput {
  name: String!
  namespace: String
  role_ref: RoleRefInput!
  subjects: [RoleBindingSubjectInput!]!
}
```