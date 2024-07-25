#!/usr/bin/env bash

[ $# -ne 1 ] && echo "Usage: $0 <service-id>" && exit 1

serviceId="--service-id=$1"

echo "Draining servicediscovery instances from $1 ..."
ids="$(aws servicediscovery list-instances $serviceId --query 'Instances[].Id' --output text | tr '\t' ' ')"

found=
for id in $ids; do
  if [ -n "$id" ]; then
    echo "Deregistering $1 / $id ..."
    aws servicediscovery deregister-instance $serviceId --instance-id "$id"
    found=1
  fi
done

# Yes, I'm being lazy here...
[ -n "$found" ] && sleep 5 || true