#!/bin/bash

set -e

# exit if we are not on linux amd64
if [ "$(uname)" != "Linux" ] || [ "$(uname -m)" != "x86_64" ]; then
  echo "This script is only supported on Linux amd64"
  exit 1
fi

# if kubectl is not installed, install it
if ! command -v kubectl &> /dev/null; then
  curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
  chmod +x kubectl
  sudo mv kubectl /usr/local/bin/kubectl
fi

# Install kind if it is not installed
if ! command -v kind &> /dev/null; then
  curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.23.0/kind-$(uname)-amd64
  chmod +x ./kind
  sudo mv ./kind /usr/local/bin/kind
fi

# Install helm if it is not installed
if ! command -v helm &> /dev/null; then
  curl -fsSL -o get_helm.sh https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3
  chmod +x get_helm.sh
  sudo ./get_helm.sh
  rm get_helm.sh
fi

kind create cluster
