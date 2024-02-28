# This base dockerfile adds all the remaining source files, performs artifact generation, and builds the project.
# See yarn-project-base/Dockerfile for deeper insight into why things are how they are.
# This should *only* build what is necessary to:
#   - Run the tests.
#   - Run the formatter checks.
# Any subsequent build steps needed to support downstream containers should be done in those containers build files.
FROM aztecprotocol/l1-contracts as contracts
FROM aztecprotocol/noir-projects as noir-projects
FROM aztecprotocol/boxes-files as boxes-files

FROM aztecprotocol/yarn-project-base
COPY --from=contracts /usr/src/l1-contracts /usr/src/l1-contracts
COPY --from=noir-projects /usr/src/noir-projects /usr/src/noir-projects
COPY --from=boxes-files /usr/src/boxes /usr/src/boxes
COPY . .
# Generate L1 contract TypeScript artifacts.
RUN cd l1-artifacts && ./scripts/generate-artifacts.sh && rm -rf /usr/src/l1-contracts
# This is actually our code generation tool. Needed to build contract typescript wrappers.
RUN yarn workspace @aztec/noir-compiler build
# Generates typescript wrappers.
RUN yarn workspace @aztec/noir-contracts.js build
# We need to build accounts as it needs to copy in account contracts from noir-contracts.
RUN yarn workspace @aztec/accounts build:copy-contracts
RUN yarn workspace @aztec/protocol-contracts build:copy-contracts
RUN yarn workspace @aztec/noir-protocol-circuits-types build
RUN yarn tsc -b
ENTRYPOINT ["yarn"]
