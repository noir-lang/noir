# Running on ubuntu until noir supports an alpine build

# Builder stage to build the noir artifacts
FROM ubuntu:kinetic

RUN apt-get update && apt-get install -y \
    curl \
    git \
    sed

WORKDIR /usr/src/yarn-project
COPY . .

WORKDIR /usr/src/yarn-project/noir-contracts 

# Download and extract nargo
RUN ./scripts/install_noir.sh
ENV PATH="/usr/src/yarn-project/noir-contracts/.nargo/bin:${PATH}"

RUN ./scripts/compile_ci.sh
