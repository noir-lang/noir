# This build image is used for launching the small docker executor in Circle CI.
# We only ever use this executor for launching powerful EC2 instances, as it's the cheapest option.
FROM alpine:latest
RUN apk add --no-cache python3 py3-pip git openssh-client ca-certificates jq yq bash curl \
    && pip3 install --upgrade pip \
    && pip3 install --no-cache-dir awscli
