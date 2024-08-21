FROM aztecprotocol/yarn-project AS yarn-project

RUN apt update && apt install socat -y

ENTRYPOINT ["/usr/src/yarn-project/cli-wallet/wallet-entrypoint.sh"]

# The version has been updated in yarn-project.
# Adding COMMIT_TAG here to rebuild versioned image.
ARG COMMIT_TAG=""
