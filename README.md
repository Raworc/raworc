<div align="center">
  <img src="assets/logo.png" alt="Raworc Logo" width="200"/>
  
  # Raworc
  
  *Remote Agent Work Orchestration*
</div>

Raworc is a cloud-native orchestration platform designed for fast AI agent deployment and user experimentation. Raworc accelerates the testing cycle by providing containerized user sessions that enable seamless agent deployment and real-world validation. The platform offers developers foundational infrastructure to rapidly deploy agents to users, gather feedback, and iterate.

The platform operates through session-based containerized environments where users interact with deployed agents. Built on Kubernetes with persistent volumes, Raworc ensures state preservation and seamless continuation, allowing users to pick up their work exactly where they left off across sessions.

Rather than running persistent agents, Raworc organizes work into discrete, manageable sessions that can be started, terminated, and remixed from any previous state, avoiding rework by building upon previous context.

## How It Works

### Session Lifecycle

1. **Session Creation**: A new work session is initiated with a starting prompt and assigned remote agents
2. **Container Deployment**: A dedicated container is spun up for the session with the specified agents
3. **Work Execution**: Agents perform their tasks within the isolated container environment
4. **Session Termination**: Sessions terminate automatically based on a predefined TTL (time-to-live) or can be manually terminated
5. **State Persistence**: Session state and data are preserved in persistent volumes upon termination
6. **Session Remix**: New sessions can be remixed from where previous sessions left off

## Key Features

- **Session-Based Architecture**: Work organized into discrete, manageable sessions
- **Kubernetes Native**: Built on Kubernetes for scalable container orchestration
- **Container Isolation**: Each session runs in its own containerized environment
- **Persistent Volume Storage**: Kubernetes persistent volumes ensure state preservation
- **Agent Assignment**: Flexible assignment of remote agents to specific sessions
- **Volume Management**: Efficient management and remixing of Kubernetes persistent volumes
- **Session Remix**: Build upon and iterate from previous sessions

## Tech Stack

- **Backend**: Rust
- **Database**: SurrealDB with TiKV engine
- **API**: GraphQL
- **Orchestration**: Kubernetes
- **Storage**: Kubernetes Persistent Volumes

## Getting Started

*coming soon*

## Installation

*coming soon*

## Usage

*coming soon*

## Scope

### In Scope

Raworc is intended to provide session-based containerized work orchestration for remote agents. As such, the project will implement or has implemented:

- Session-based containerized work environments
- Remote agent orchestration and management
- Persistent volume management for session state
- Session remixing and continuation capabilities
- Kubernetes-native deployment and scaling
- GraphQL API for flexible data access

### Out of Scope

Raworc will be used in a cloud native environment with other tools. The following specific functionality will therefore not be incorporated:

- Direct agent execution outside of containerized sessions
- Real-time streaming of agent outputs
- Agent development frameworks or tooling

## Communications

- X: [@raworc](http://x.com/raworc)

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
