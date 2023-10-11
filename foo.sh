#!/usr/bin/env bash
set eu

function say() {
  printf 'bb: %s\n' "$1"
}

function err() {
  say "$1" >&2
  exit 1
}

echo "I'm installing a backend"

echo "I'm running ${NARGO_OS}"
echo "My architecture is ${NARGO_ARCHITECTURE}"

echo "The backend will be installed to ${NARGO_BACKEND_DESTINATION_PATH}"

case $NARGO_OS in
  linux)
    PLATFORM="linux-gnu"
    ;;
  darwin)
    PLATFORM="apple-darwin"
    ;;
  *)
    err "unsupported platform: $PLATFORM"
    ;;
esac

# Reject unsupported architectures.
if [ "${NARGO_ARCHITECTURE}" == "aarch64" ] && [ "${PLATFORM}" == "linux-gnu" ] || [ "${NARGO_ARCHITECTURE}" != "x86_64" ] && [ "${NARGO_ARCHITECTURE}" != "aarch64" ]; then
  err "unsupported architecure: $NARGO_ARCHITECTURE-$PLATFORM"
fi

USERNAME="AztecProtocol"
REPO="aztec-packages"
VERSION="0.7.3"
TAG="aztec-packages-v${VERSION}"

RELEASE_URL="https://github.com/${USERNAME}/${REPO}/releases/download/${TAG}/barretenberg-${NARGO_ARCHITECTURE}-${PLATFORM}.tar.gz"
echo "Installing from $RELEASE_URL";