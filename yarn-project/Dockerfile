# This base dockerfile adds all the remaining source files, performs artifact generation, and builds the project.
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/noir-contracts-build as noir
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base

# Copy in the entire workspace.
COPY . .

# Generate Noir contract TypeScript artifacts.
COPY --from=noir /usr/src/yarn-project/noir-contracts/target /usr/src/yarn-project/noir-contracts/target
# Run yarn build to have the json ABIs available for the types generator, generate types, build again.
RUN cd /usr/src/yarn-project/noir-contracts && yarn build && ./scripts/types_all.sh && yarn build

# Build the entire project and check formatting.
RUN yarn tsc -b
ENTRYPOINT [ "yarn" ]
