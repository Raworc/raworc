# TiKV + SurrealDB Kubernetes Configuration

This directory contains all configuration files for deploying TiKV and SurrealDB on Kubernetes.

## Architecture

```
SurrealDB (with GraphQL) → TiKV Cluster → Persistent Storage
```

## Directory Structure

```
kube/
├── deploy.sh                      # Deployment script
├── uninstall.sh                   # Uninstall script
├── tikv/                          # TiKV cluster configurations
│   ├── tikv-cluster.yaml          # Main TiKV cluster deployment
│   ├── tikv-external-service.yaml # LoadBalancer service
│   └── tikv-nodeport-service.yaml # NodePort service
└── surrealdb/                     # SurrealDB configurations
    ├── surrealdb-deployment.yaml  # SurrealDB deployment
    ├── surrealdb-service.yaml     # ClusterIP & NodePort services
    ├── surrealdb-lb.yaml          # LoadBalancer service
    └── surrealdb-ingress.yaml     # Ingress configuration
```

## Prerequisites

1. Kubernetes cluster (tested on v1.33.1)
2. TiDB Operator v1.6.0
3. Storage class for persistent volumes

## Installation Steps

### 1. Install TiDB Operator

```bash
helm repo add pingcap https://charts.pingcap.org/
helm repo update
helm install tidb-operator pingcap/tidb-operator \
  --namespace tidb-admin \
  --create-namespace \
  --version v1.6.0
```

### 2. Deploy TiKV Cluster

```bash
kubectl create namespace tikv
kubectl apply -f tikv/tikv-cluster.yaml
kubectl apply -f tikv/tikv-external-service.yaml
kubectl apply -f tikv/tikv-nodeport-service.yaml
```

### 3. Deploy SurrealDB

```bash
kubectl create namespace surrealdb
kubectl apply -f surrealdb/surrealdb-deployment.yaml
kubectl apply -f surrealdb/surrealdb-service.yaml
kubectl apply -f surrealdb/surrealdb-lb.yaml
kubectl apply -f surrealdb/surrealdb-ingress.yaml
```

## Configuration Details

### TiKV Cluster
- **Version**: v7.5.0
- **PD Replicas**: 3 (10Gi storage each)
- **TiKV Replicas**: 3 (20Gi storage each)
- **TiDB Replicas**: 0 (disabled, only using TiKV)
- **Total Storage**: 90Gi

### SurrealDB
- **Version**: 2.3.7
- **Storage Backend**: TiKV (tikv://tikv-cluster-pd.tikv:2379)
- **Features**: GraphQL enabled (experimental)
- **Authentication**: root/root

## Access Endpoints

### External Access (Replace with your actual IPs)
- **SurrealDB LoadBalancer**: `http://<EXTERNAL-IP>:8000`
- **SurrealDB NodePort**: `http://<NODE-IP>:32000`
- **TiKV PD LoadBalancer**: `<EXTERNAL-IP>:2379`
- **TiKV PD NodePort**: `<NODE-IP>:32379`

### Internal Access
- **SurrealDB**: `http://surrealdb.surrealdb:8000`
- **TiKV PD**: `tikv-cluster-pd.tikv:2379`

## GraphQL Configuration

GraphQL is enabled as an experimental feature. To use it:

1. Create namespace and database:
```bash
curl -X POST http://<SURREALDB-IP>:8000/sql \
  -u root:root \
  -H "Accept: application/json" \
  -H "Surreal-NS: test" \
  -H "Surreal-DB: test" \
  -d "DEFINE CONFIG GRAPHQL AUTO;"
```

2. Query via GraphQL:
```bash
curl -X POST http://<SURREALDB-IP>:8000/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic $(echo -n 'root:root' | base64)" \
  -H "Surreal-NS: test" \
  -H "Surreal-DB: test" \
  -d '{"query": "{ __schema { types { name } } }"}'
```

## Monitoring

Check cluster status:
```bash
# TiKV cluster status
kubectl get tidbcluster -n tikv
kubectl get pods -n tikv

# SurrealDB status
kubectl get pods -n surrealdb
kubectl logs -n surrealdb deployment/surrealdb
```

## Troubleshooting

1. **Pods not starting**: Check node resources with `kubectl describe node`
2. **Connection issues**: Verify services with `kubectl get svc -A | grep -E "(tikv|surreal)"`
3. **GraphQL errors**: Ensure namespace/database exist and GraphQL is configured

## Security Notes

- GraphQL is a pre-release feature with security warnings
- Default credentials (root/root) should be changed for production
- Consider using network policies to restrict access