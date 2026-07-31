#!/usr/bin/env bash

set -euo pipefail

k3d cluster delete lgtm-cluster || true

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
helm repo add grafana-community https://grafana-community.github.io/helm-charts
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

echo "  Deploying Grafana..."
# helm upgrade --install grafana oci://ghcr.io/grafana-community/helm-charts/grafana
helm upgrade --install grafana grafana/grafana \
  --namespace lgtm \
  -f /mnt/manifests/grafana-values.yaml \
  --set "grafana.ini.server.domain=grafana.lgtm.local" \
  --set "grafana.ini.server.root_url=http://grafana.lgtm.local:8080" \
  --set "grafana.ini.server.serve_from_sub_path=false"

echo "  Deploying Loki..."
# helm upgrade --install loki oci://ghcr.io/grafana-community/helm-charts/loki
helm upgrade --install loki grafana/loki \
  --namespace lgtm \
  --set deploymentMode=SingleBinary \
  --set singleBinary.replicas=1 \
  --set backend.replicas=0 \
  --set read.replicas=0 \
  --set write.replicas=0 \
  --set loki.auth_enabled=false \
  --set loki.commonConfig.replication_factor=1 \
  --set loki.storage.type=filesystem \
  --set loki.useTestSchema=true

echo "  Deploying Tempo..."
# helm upgrade --install tempo oci://ghcr.io/grafana-community/helm-charts/tempo
helm upgrade --install tempo grafana/tempo \
  --namespace lgtm

echo "  Deploying Prometheus..."
helm upgrade --install prometheus prometheus-community/prometheus \
  --namespace lgtm \
  --set alertmanager.enabled=false \
  --set server.persistentVolume.enabled=false \
  --set pushgateway.enabled=false \
  --set server.extraFlags[0]="web.enable-remote-write-receiver" \
  --set server.extraFlags[1]="enable-feature=exemplar-storage"

echo "  Deploying Grafana Alloy..."
helm upgrade --install alloy grafana/alloy \
  --namespace lgtm \
  --set alloy.clustering.enabled=false \
  --set alloy.enableReporting=false \
  --set "alloy.extraPorts[0].name=otlp-grpc" \
  --set "alloy.extraPorts[0].port=4317" \
  --set "alloy.extraPorts[0].targetPort=4317" \
  --set "alloy.extraPorts[0].protocol=TCP" \
  --set-file alloy.configMap.content=/mnt/tools/alloy-config.river

echo "  Deploying app..."
kubectl apply -f /mnt/manifests/deployment-backend.yaml
kubectl apply -f /mnt/manifests/deployment-frontend.yaml
kubectl apply -f /mnt/manifests/deployment-ipfs.yaml
kubectl apply -f /mnt/manifests/ingress.yaml

echo "Cluster deployment complete!"
