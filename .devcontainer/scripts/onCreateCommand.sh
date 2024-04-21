#!/usr/bin/env bash

TYPE=$1
NAME=$2


curl -s install.aztec.network | NON_INTERACTIVE=1 BIN_PATH=/usr/local/bin bash -s
docker compose -f $HOME/.aztec/docker-compose.yml pull

if ! grep -q "PXE_URL" ~/.bashrc; then
    echo "export PXE_URL=https://\$CODESPACE_NAME-8080.preview.\$GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN" >> ~/.bashrc
fi

yes | npx create-aztec-app -t $TYPE -n $NAME -s
mv react/* react/.* .
rm -rf react

yarn

npx -y playwright install --with-deps
yarn prep
