#!/usr/bin/env bash

TYPE=$1
NAME=$2

curl -s install.aztec.network | NON_INTERACTIVE=1 BIN_PATH=/usr/local/bin bash -s
docker compose -f $HOME/.aztec/docker-compose.yml pull

if ! grep -q "PXE_URL" ~/.bashrc; then
    echo "export PXE_URL=https://\$CODESPACE_NAME-8080.preview.\$GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN" >> ~/.bashrc
fi

if ! grep -q "alias sandbox" ~/.bashrc; then
    echo "alias sandbox=\"npx aztec-app sandbox\"" >> ~/.bashrc
fi
corepack enable

if [ "$TYPE" != "sandbox_only" ]; then
    source ~/.bashrc
    yes | npx create-aztec-app -t $TYPE -n $NAME -s
    mv $NAME/* $NAME/.* .
    rm -rf $NAME
    
    yarn

    npx -y playwright install --with-deps
    yarn add @aztec/builder
    yarn prep
fi
