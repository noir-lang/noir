# Builds the boxes (they were copied into yarn-project-base so the cli can unbox).
# Produces a container that can be run to test a specific box. See docker-compose.yml.
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/aztec AS aztec
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/noir as noir

# We need yarn. Start fresh container.
FROM node:18.19.0
COPY --from=aztec /usr/src /usr/src
COPY --from=noir /usr/src/noir/target/release/nargo /usr/src/noir/target/release/nargo
WORKDIR /usr/src/boxes
ENV AZTEC_NARGO=/usr/src/noir/target/release/nargo
ENV AZTEC_CLI=/usr/src/yarn-project/cli/aztec-cli-dest
RUN yarn && yarn build
ENTRYPOINT ["yarn", "workspace"]
