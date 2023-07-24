FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/barretenberg-x86_64-linux-clang-assert
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/bb.js

FROM node:18-alpine
RUN apk update && apk add git bash curl
COPY --from=0 /usr/src/barretenberg/cpp/build /usr/src/barretenberg/cpp/build
COPY --from=1 /usr/src/barretenberg/ts /usr/src/barretenberg/ts
WORKDIR /usr/src/barretenberg/acir_tests
COPY . .
# Run all native tests.
RUN ./run_acir_tests.sh
# Just run double_verify_proof as a sanity check as bb.js is quite slow.
RUN BB=./run_bb.sh ./run_acir_tests.sh double_verify_proof