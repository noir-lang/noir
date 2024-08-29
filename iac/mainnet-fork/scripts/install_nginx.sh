#!/bin/sh
set -eu

# See https://nginx.org/en/linux_packages.html#Ubuntu
echo 'debconf debconf/frontend select Noninteractive' | debconf-set-selections

apt-get update && apt install -y curl gnupg2 ca-certificates lsb-release ubuntu-keyring

curl -sS https://nginx.org/keys/nginx_signing.key | gpg --dearmor \
    | tee /usr/share/keyrings/nginx-archive-keyring.gpg >/dev/null

echo "deb [signed-by=/usr/share/keyrings/nginx-archive-keyring.gpg] \
    http://nginx.org/packages/ubuntu `lsb_release -cs` nginx" \
    | tee /etc/apt/sources.list.d/nginx.list

echo "Package: *\nPin: origin nginx.org\nPin: release o=nginx\nPin-Priority: 900\n" \
    | tee /etc/apt/preferences.d/99nginx

apt-get update && apt install -y git curl nginx nginx-module-njs