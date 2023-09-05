FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

ARG DOCKER_TAG

# Remove SRS files (currently not producing proofs)
RUN rm -rf /usr/src/circuits/cpp/barretenberg/cpp/srs_db/ignition/monomial
COPY . .

# Update aztec-rpc version if DOCKER_TAG has been used
WORKDIR /usr/src/yarn-project/aztec-rpc
RUN if [[ -n "${DOCKER_TAG}" ]]; then \
  jq --arg v ${DOCKER_TAG} '.version = $v' package.json > _temp && mv _temp package.json; \
  fi

WORKDIR /usr/src/yarn-project/aztec-sandbox
RUN ls
RUN if [[ -n "${DOCKER_TAG}" ]]; then \
  jq --arg v ${DOCKER_TAG} '.version = $v' package.json > _temp && mv _temp package.json; \
  fi

RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine

COPY --from=builder /usr/src/ /usr/src/
WORKDIR /usr/src/yarn-project/aztec-sandbox

ENTRYPOINT ["yarn"]
CMD [ "start" ]
EXPOSE 8080