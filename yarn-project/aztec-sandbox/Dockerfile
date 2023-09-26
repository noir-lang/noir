FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project AS builder

ARG COMMIT_TAG=""

# Remove SRS files (currently not producing proofs)
RUN rm -rf /usr/src/barretenberg/cpp/srs_db/ignition/monomial
COPY . .

# Update pxe version if COMMIT_TAG has been used
WORKDIR /usr/src/yarn-project/pxe
RUN if [[ -n "${COMMIT_TAG}" ]]; then \
  jq --arg v ${COMMIT_TAG} '.version = $v' package.json > _temp && mv _temp package.json; \
  fi

WORKDIR /usr/src/yarn-project/aztec-sandbox
RUN if [[ -n "${COMMIT_TAG}" ]]; then \
  jq --arg v ${COMMIT_TAG} '.version = $v' package.json > _temp && mv _temp package.json; \
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
