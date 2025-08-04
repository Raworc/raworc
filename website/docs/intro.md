---
sidebar_position: 1
title: Introduction
slug: /
---

# Welcome to Raworc

<div align="center">
  <img src="/img/logo.png" alt="Raworc Logo" width="200"/>
</div>

**Raworc** is a cloud-native orchestration platform designed for fast AI agent deployment and user experimentation. It accelerates the testing cycle by providing containerized user sessions that enable seamless agent deployment and real-world validation.

## What is Raworc?

Raworc (Remote Agent Work Orchestration) provides developers with foundational infrastructure to rapidly deploy agents to users, gather feedback, and iterate. The platform operates through session-based containerized environments where users interact with deployed agents.

### Key Features

- 🚀 **Session-Based Architecture** - Work organized into discrete, manageable sessions
- 🔧 **Kubernetes Native** - Built on Kubernetes for scalable container orchestration
- 🔒 **Container Isolation** - Each session runs in its own containerized environment
- 💾 **Persistent State** - Kubernetes persistent volumes ensure state preservation
- 🤖 **Flexible Agent Assignment** - Assign multiple agents to specific sessions
- 🔄 **Session Remixing** - Build upon and iterate from previous sessions
- 🛡️ **RBAC System** - Fine-grained role-based access control
- 📊 **REST API** - Comprehensive API with OpenAPI documentation

## Why Raworc?

Traditional agent deployment approaches often struggle with:
- Long iteration cycles between development and user testing
- Difficulty preserving context across multiple work sessions
- Complex infrastructure requirements for isolated agent execution
- Managing state and continuity in distributed environments

Raworc solves these challenges by:
- Providing instant containerized environments for each work session
- Automatically preserving state through persistent volumes
- Enabling users to remix and continue from any previous session
- Offering a simple REST API for seamless integration

## Quick Links

- [Quick Start Guide](/docs/getting-started/quickstart) - Get running in 5 minutes
- [Architecture Overview](/docs/concepts/architecture) - Understand how Raworc works
- [CLI Examples](/docs/guides/cli-examples) - Common CLI usage patterns
- [REST API Reference](/docs/api/rest-api) - Complete API documentation

## Getting Help

- 📚 Browse the [documentation](/docs/intro)
- 🐛 Report issues on [GitHub](https://github.com/raworc/raworc/issues)
- 🐦 Follow us on [X/Twitter](https://x.com/raworc)