*This project has been created as part of the 42 curriculum by mcutura*

# ft_lgtm

## Description

Looks Good To Monitor (LGTM) is as an observability stack for monitoring and alerting disguised as a code compiling web app.  
It is built using Loki, Grafana, Tempo, Prometheus and Alloy, deployed in a k3d cluster inside a libvirt VM.  

## Features

- Rust code compilation to WASM and execution in the sandbox
- Store and share code snippets on IPFS (InterPlanetary File System)
- Observability stack for monitoring and alerting

## Requirements

- libvirt and virt-manager
```
brew install libvirt virt-manager
brew services start libvirt
```

## Instructions

`make install` for the first run  
`make start` to launch the full stack  

## Usage

## Technical choices

## Resources
