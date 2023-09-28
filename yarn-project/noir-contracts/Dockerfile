FROM ubuntu:lunar
RUN apt-get update && apt-get install -y \
    jq \
    curl \
    git \
    sed
WORKDIR /usr/src/yarn-project
COPY noir-contracts noir-contracts
COPY noir-compiler noir-compiler
COPY aztec-nr aztec-nr
WORKDIR /usr/src/yarn-project/noir-contracts

# Download and extract nargo
ENV NARGO_HOME="/usr/src/yarn-project/noir-contracts/.nargo"
RUN ./scripts/install_noirup.sh $(pwd)
ENV PATH="/usr/src/yarn-project/noir-contracts/.nargo/bin:${PATH}"

RUN ./scripts/install_noir.sh
RUN ./scripts/compile_all.sh
RUN ./scripts/nargo_test_ci.sh
