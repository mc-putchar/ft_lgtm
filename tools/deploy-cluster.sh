#!/usr/bin/env bash

set -euo pipefail

echo "  Creating k3d cluster..."
k3d cluster create lgtm-cluster \
  --registry-create lgtm-registry:0.0.0.0:5001 \
  --port "80:80@loadbalancer" \
  --port "443:443@loadbalancer" \
  --volume "/mnt:/mnt@all"

echo "  Waiting for cluster to be ready..."
kubectl cluster-info

kubectl apply -f /mnt/manifests/namespaces.yaml

echo "  Deploying LGTM stack..."
helm repo add grafana https://grafana.github.io/helm-charts
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

echo "  Deploying Prometheus..."
helm upgrade --install prometheus prometheus-community/prometheus \
  --namespace lgtm \
  --set alertmanager.enabled=false \
  --set server.persistentVolume.enabled=false \
  --set pushgateway.enabled=false

echo "  Deploying Loki (Single Binary Mode)..."
helm upgrade --install loki grafana/loki \
  --namespace lgtm \
  --set deploymentMode=SingleBinary \
  --set loki.auth_enabled=false \
  --set loki.commonConfig.replication_factor=1 \
  --set singleBinary.replicas=1

echo "  Deploying Tempo..."
helm upgrade --install tempo grafana/tempo \
  --namespace lgtm

echo "  Deploying Grafana..."
helm upgrade --install grafana grafana/grafana \
  --namespace lgtm

echo "  Deploying Grafana Alloy..."
helm upgrade --install alloy grafana/alloy \
  --namespace lgtm \
  --set alloy.clustering.enabled=false \
  --set alloy.enableReporting=false


echo "  Deploying app..."
kubectl apply -f /mnt/manifests/deployment-backend.yaml
kubectl apply -f /mnt/manifests/deployment-frontend.yaml
kubectl apply -f /mnt/manifests/deployment-ipfs.yaml
kubectl apply -f /mnt/manifests/ingress.yaml

echo "Cluster deployment complete!"
