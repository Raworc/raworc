#!/bin/bash

# TiKV + SurrealDB Deployment Script

set -e

echo "Starting TiKV + SurrealDB deployment..."

# Check if namespaces exist
if ! kubectl get namespace tikv &> /dev/null; then
    echo "Creating tikv namespace..."
    kubectl create namespace tikv
fi

if ! kubectl get namespace surrealdb &> /dev/null; then
    echo "Creating surrealdb namespace..."
    kubectl create namespace surrealdb
fi

# Deploy TiKV
echo "Deploying TiKV cluster..."
kubectl apply -f tikv/tikv-cluster.yaml
kubectl apply -f tikv/tikv-external-service.yaml
kubectl apply -f tikv/tikv-nodeport-service.yaml

# Wait for TiKV PD to be ready
echo "⏳ Waiting for TiKV PD pods to be ready..."
kubectl wait --for=condition=ready pod -l app.kubernetes.io/component=pd -n tikv --timeout=300s

# Deploy SurrealDB
echo "Deploying SurrealDB..."
kubectl apply -f surrealdb/surrealdb-deployment.yaml
kubectl apply -f surrealdb/surrealdb-service.yaml
kubectl apply -f surrealdb/surrealdb-lb.yaml
kubectl apply -f surrealdb/surrealdb-ingress.yaml

# Wait for SurrealDB to be ready
echo "⏳ Waiting for SurrealDB to be ready..."
kubectl wait --for=condition=ready pod -l app=surrealdb -n surrealdb --timeout=120s

# Get access endpoints
echo ""
echo "Deployment complete!"
echo ""
echo "Cluster Status:"
kubectl get pods -n tikv
echo ""
kubectl get pods -n surrealdb
echo ""

echo "External Access Points:"
echo "SurrealDB Services:"
kubectl get svc -n surrealdb | grep -E "(LoadBalancer|NodePort)"
echo ""
echo "TiKV Services:"
kubectl get svc -n tikv | grep -E "(LoadBalancer|NodePort)"

echo ""
echo "To test SurrealDB connection:"
echo "curl http://<EXTERNAL-IP>:8000/"