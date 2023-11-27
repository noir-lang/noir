# This base dockerfile adds all the remaining source files, performs artifact generation, and builds the project.
# See yarn-project-base/Dockerfile for deeper insight into why things are how they are.
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base as builder

RUN apk add bash perl

# Copy in the entire workspace.
COPY . .

RUN yarn workspace @aztec/foundation build && \
  yarn workspace @aztec/noir-compiler build && \
  yarn workspace @aztec/noir-contracts noir:build:all && \
  yarn workspace @aztec/noir-protocol-circuits noir:build && \
  yarn tsc -b

# Build aztec.js web artifact
RUN cd /usr/src/yarn-project/aztec.js && yarn build:web

FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base
COPY --from=builder /usr/src/yarn-project /usr/src/yarn-project

WORKDIR /usr/src/yarn-project

ENTRYPOINT ["yarn"]
