*This project has been created as part of the 42 curriculum by mcutura*

# ft_lgtm

## Description

Looks Good To Monitor (LGTM) is as an observability stack for monitoring and alerting hidden behind a code execution web app.  
It is built using Loki, Grafana, Tempo, Prometheus and Alloy, deployed in a k3d cluster inside a libvirt VM.  

## Features

- Rust code compilation to WASM and execution in the WASI sandbox
- Store and share code snippets on IPFS (InterPlanetary File System)
- Observability stack for monitoring and alerting

## Requirements

- libvirt and virt-manager
```
brew install libvirt virt-manager
brew services start libvirt
```

## Instructions

`make auto` for automated setup and deploy (approx ~5-10 mins)  
`make help` to display all available commands  

## Usage

Get Grafana admin password:
```sh
kubectl -n lgtm get secret grafana -o jsonpath="{.data.admin-password}" | base64 -d; echo
```

Forward Kubo IPFS WebUI to host machine:
```sh
kubectl exec -n app deploy/ipfs -- ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["http://127.0.0.1:5002", "http://localhost:5002", "http://127.0.0.1:5001"]'
kubectl exec -n app deploy/ipfs -- ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["PUT", "POST", "GET"]'
kubectl -n app port-forward --address 0.0.0.0 svc/ipfs-service 5002:5001
```

## Technical choices

## Resources
