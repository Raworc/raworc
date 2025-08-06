---
sidebar_position: 1
title: Architecture Overview
---

# Architecture Overview

Raworc is built as a cloud-native platform designed for scalability, reliability, and flexibility. This document provides a comprehensive overview of the system architecture.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Control Plane                           │
├─────────────────────┬───────────────────────────────────────────┤
│   Raworc CLI        │         Raworc Service (REST API)         │
│  (User Interface)   │    ┌─────────────┬──────────────────┐     │
│                     │    │   Auth      │   RBAC System    │     │
│                     │    │   Module    │   (Roles/Perms)  │     │
│                     │    └─────────────┴──────────────────┘     │
└─────────────────────┴───────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                          Data Layer                             │
│                    PostgreSQL Database                          │
│  ┌──────────────┬────────────────┬─────────────────────────┐    │
│  │ Service      │     Roles &    │       Agents &          │    │
│  │ Accounts     │  Role Bindings │      Sessions           │    │
│  └──────────────┴────────────────┴─────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Session Runtime                            │
│                  (Kubernetes Cluster)                           │
│  ┌────────────────────────────────────────────────────────┐     │
│  │              Container Per Session                     │     │
│  │  ┌────────────┬─────────────┬────────────────────┐     │     │
│  │  │Guardrails  │   Agents    │  Persistent Volume │     │     │
│  │  │& Safety    │  (Built-in  │    (Session State) │     │     │
│  │  │            │ & External) │                    │     │     │
│  │  └────────────┴─────────────┴────────────────────┘     │     │
│  └────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Control Plane

The control plane manages the entire Raworc system and consists of:

#### Raworc CLI (`raworc`)

- Command-line interface for users and developers
- Supports interactive mode with auto-completion
- Handles authentication and session management
- Provides direct API access through `/api` commands

#### Raworc Service

- REST API server written in Rust
- Handles all platform operations
- Manages authentication via JWT tokens
- Implements RBAC for fine-grained access control
- Provides OpenAPI/Swagger documentation

### 2. Data Layer

PostgreSQL serves as the primary database, storing:

#### Service Accounts

- Internal authentication entities
- Username/password credentials
- Namespace scoping
- Active/inactive status tracking

#### RBAC System

- **Roles**: Define permissions through rules
- **Role Bindings**: Connect roles to principals
- **Rules**: Specify API groups, resources, and verbs

#### Agents

- Agent configurations and metadata
- Model specifications
- Tool and guardrail assignments
- Routing patterns
- Knowledge base references

#### Sessions (Future)

- Session metadata and configuration
- Agent assignments
- Resource allocations
- State references

### 3. Session Runtime

The Kubernetes-based runtime environment provides:

#### Container Isolation

- Each session runs in its own container
- Complete resource isolation
- Security boundaries
- Custom runtime environments

#### Persistent Volumes

- State preservation across session restarts
- Data continuity for remixing
- Efficient storage management
- Backup and recovery capabilities

#### Agent Execution

- Built-in agent runtime (`raworc-agent`)
- External agent integration
- Tool execution environment
- Guardrail enforcement

## Key Design Principles

### 1. Session-Based Work

Unlike traditional persistent agent systems, Raworc organizes work into discrete sessions:

- **Ephemeral Execution**: Sessions have defined lifecycles
- **State Preservation**: All work is saved to persistent volumes
- **Remixability**: New sessions can continue from any previous state
- **Resource Efficiency**: Resources are allocated only when needed

### 2. Cloud-Native Architecture

Built for modern cloud environments:

- **Kubernetes Native**: Leverages Kubernetes for orchestration
- **Horizontal Scalability**: Add more nodes as needed
- **High Availability**: Multiple replicas and failover
- **Container-First**: Everything runs in containers

### 3. Security First

Multiple layers of security:

- **JWT Authentication**: Secure token-based auth
- **RBAC**: Fine-grained permission control
- **Container Isolation**: Process and network isolation
- **Audit Logging**: Comprehensive activity tracking

## Data Flow

### 1. Session Creation Flow

```
User → CLI → REST API → RBAC Check → Database → Kubernetes API → Container
```

1. User initiates session via CLI
2. CLI sends request to REST API
3. RBAC system validates permissions
4. Session metadata stored in database
5. Kubernetes creates container
6. Container initialized with agents

### 2. Agent Execution Flow

```
Container → Agent Runtime → Guardrails → Tools → Results → Persistent Volume
```

1. Container receives work request
2. Agent runtime processes request
3. Guardrails validate safety
4. Tools execute actions
5. Results returned to user
6. State saved to persistent volume

### 3. Session Remix Flow

```
User → Select Previous Session → Copy PV → New Container → Continue Work
```

1. User selects previous session
2. System copies persistent volume
3. New container created with copied state
4. Work continues from previous point

## Scalability Considerations

### Horizontal Scaling

- **API Servers**: Multiple replicas behind load balancer
- **Database**: Read replicas for query distribution
- **Session Containers**: Unlimited parallel sessions

### Vertical Scaling

- **Container Resources**: Configurable CPU/memory per session
- **Persistent Volumes**: Expandable storage capacity
- **Database**: Upgradeable instance sizes

### Performance Optimization

- **Connection Pooling**: Efficient database connections
- **Caching**: Redis integration (future)
- **Lazy Loading**: On-demand resource allocation
- **Batch Operations**: Efficient bulk processing

## Integration Points

### External Systems

- **CI/CD Pipelines**: API integration for automation
- **Monitoring**: Prometheus metrics export
- **Logging**: Structured logs for analysis
- **Storage**: S3-compatible object storage

### Agent Integration

- **LLM Providers**: OpenAI, Anthropic, etc.
- **Tool Services**: External API calls
- **Knowledge Bases**: Vector databases
- **Custom Agents**: Proprietary implementations

## Future Architecture Evolution

### Planned Enhancements

- **Multi-Region**: Geographic distribution
- **Edge Deployment**: Local execution capabilities
- **Streaming**: Real-time session updates
- **Federation**: Multi-cluster coordination

### Extensibility

- **Plugin System**: Custom extensions
- **Webhook Support**: Event notifications
- **Custom Resources**: Kubernetes CRDs
- **API Extensions**: Versioned endpoints

