# Builds the boxes (they were copied into yarn-project-base so the cli can unbox).
# Produces a container that can be run to test a specific box. See docker-compose.yml.
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/aztec-sandbox AS aztec-sandbox

# We need yarn. Start fresh container.
FROM node:18.19.0
COPY --from=aztec-sandbox /usr/src /usr/src
WORKDIR /usr/src/boxes
ENV AZTEC_CLI=../../yarn-project/cli/aztec-cli-dest
RUN yarn && yarn build
ENTRYPOINT ["yarn", "workspace"]