# Raworc Build Command

The `raworc build` command allows you to build Docker images for different Raworc components directly from the CLI.

## Usage

```bash
# Build all images
raworc build

# Build specific components
raworc build server
raworc build operator host
raworc build server operator host

# Build with custom tag
raworc build --tag v1.0.0

# Build without cache
raworc build --no-cache

# Build and push to registry
raworc build --push --registry docker.io/myorg
```

## Components

- `server` - API server that handles REST requests
- `operator` - Session manager that handles Docker containers
- `host` - Host agent that runs inside containers
- `all` - Build all components (default if no component specified)

## Options

- `--tag, -t <TAG>` - Docker image tag (default: latest)
- `--no-cache` - Build without using Docker cache
- `--push, -p` - Push images to registry after building
- `--registry, -r <REGISTRY>` - Registry to push to (e.g., docker.io/myorg)

## Examples

```bash
# Build only server with custom tag
raworc build server --tag v2.0.0

# Build all and push to Docker Hub
raworc build --push --registry docker.io/raworc

# Build operator without cache
raworc build operator --no-cache

# Build multiple components
raworc build server operator --tag production
```

## Image Names

The following images will be created:
- `raworc-server:<tag>`
- `raworc-operator:<tag>`
- `raworc-host:<tag>`

When pushing to a registry, images will be tagged as:
- `<registry>/raworc-server:<tag>`
- `<registry>/raworc-operator:<tag>`
- `<registry>/raworc-host:<tag>`