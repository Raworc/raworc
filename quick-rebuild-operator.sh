#!/bin/bash
set -e

echo "Building raworc binary..."
cargo build --release

echo "Creating minimal operator image..."
cat > /tmp/Dockerfile.operator-quick << 'EOF'
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 libpq5 && rm -rf /var/lib/apt/lists/*
COPY target/release/raworc /usr/local/bin/raworc
CMD ["raworc", "operator"]
EOF

docker build -f /tmp/Dockerfile.operator-quick -t raworc-operator:latest .

echo "Restarting operator..."
docker compose restart raworc-operator

echo "Done!"