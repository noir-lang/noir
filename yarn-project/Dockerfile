# This base dockerfile adds all the remaining source files, performs artifact generation, and builds the project.
# See yarn-project-base/Dockerfile for deeper insight into why things are how they are.
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/noir-contracts-build as noir
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base as builder

# Copy in the entire workspace.
COPY . .

# Generate Noir contract TypeScript artifacts.
COPY --from=noir /usr/src/yarn-project/noir-contracts/target /usr/src/yarn-project/noir-contracts/target
# Run yarn build to have the json ABIs available for the types generator, generate types, build again.
RUN apk add perl
RUN cd /usr/src/yarn-project/noir-contracts && yarn build && ./scripts/types_all.sh && yarn build
# Cleanup to reduce final image size.
RUN rm -rf noir-contracts/target

# Build the entire project.
RUN yarn tsc -b

FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base
COPY --from=builder /usr/src/yarn-project /usr/src/yarn-project

# Download minimal CRS requirements
COPY --from=builder /usr/src/barretenberg/cpp/srs_db/download_ignition.sh /usr/src/yarn-project/circuits.js/resources/download_ignition.sh
WORKDIR /usr/src/yarn-project/circuits.js/resources
RUN ./download_ignition.sh 0 28 3200027
RUN ./download_ignition.sh 0 322560092 322560219 true

WORKDIR /usr/src/yarn-project

ENTRYPOINT ["yarn"]