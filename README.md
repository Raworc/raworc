<div align="center">
  <img src="assets/logo.png" alt="Raworc Logo" width="200"/>
  
  # Raworc
  
  *Remote Agent Work Orchestration*
  
  [🌐 Website](https://raworc.com) | [📚 Documentation](https://raworc.com/docs) | [🐦 Twitter](https://twitter.com/raworc)
</div>

Raworc is a cloud-native orchestration platform for fast AI agent deployment and user experimentation through containerized sessions.

## Quick Start

```bash
# Clone the repository
git clone https://github.com/Raworc/raworc.git
cd raworc

# Build and run
cargo build --release
./target/release/raworc start

# In another terminal, authenticate
./target/release/raworc auth

# Connect to the CLI (default endpoint: http://localhost:8080)
./target/release/raworc --endpoint http://localhost:8080
```

## Documentation

Full documentation is available at [raworc.com](https://raworc.com):

- [Getting Started](https://raworc.com/docs/getting-started/quickstart)
- [Architecture](https://raworc.com/docs/concepts/architecture)
- [API Reference](https://raworc.com/docs/api/rest-api)
- [Configuration](https://raworc.com/docs/admin/configuration)
- [RBAC System](https://raworc.com/docs/admin/rbac)

## Live API Documentation

When the server is running:
- **Swagger UI**: `/swagger-ui/`
- **OpenAPI Spec**: `/api-docs/openapi.json`

## License

Apache License 2.0 - see the [LICENSE](LICENSE) file for details.