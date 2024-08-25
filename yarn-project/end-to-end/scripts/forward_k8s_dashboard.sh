#!/bin/bash
set -e
# spit out the token for the dashboard
kubectl -n kubernetes-dashboard create token admin-user --duration 300000m
# forward the dashboard
kubectl -n kubernetes-dashboard port-forward svc/kubernetes-dashboard-kong-proxy 8443:443
