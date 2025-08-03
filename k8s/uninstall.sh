#!/bin/bash

# TiKV + SurrealDB Uninstall Script

echo "Uninstalling TiKV + SurrealDB..."

# Delete SurrealDB
echo "Removing SurrealDB..."
kubectl delete -f surrealdb/ -n surrealdb 2>/dev/null || true

# Delete TiKV
echo "Removing TiKV cluster..."
kubectl delete -f tikv/ -n tikv 2>/dev/null || true

# Optional: Delete namespaces (this will also delete PVCs)
read -p "Delete namespaces and persistent data? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    kubectl delete namespace surrealdb
    kubectl delete namespace tikv
    echo "Namespaces and data deleted"
else
    echo "WARNING: Namespaces kept. PVCs still exist."
    echo "To manually delete: kubectl delete namespace surrealdb tikv"
fi

echo "Uninstall complete!"