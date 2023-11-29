# This base dockerfile adds all the remaining source files, performs artifact generation, and builds the project.
# See yarn-project-base/Dockerfile for deeper insight into why things are how they are.
# This should *only* build what is necessary to:
#   - Run the tests.
#   - Run the formatter checks.
# Any subsequent build steps needed to support downstream containers should be done in those containers build files.
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base as builder

RUN apk add bash perl

# Copy in the entire workspace.
COPY . .

RUN yarn workspace @aztec/foundation build && \
  yarn workspace @aztec/noir-compiler build && \
  yarn workspace @aztec/noir-contracts noir:build:all && \
  yarn workspace @aztec/noir-protocol-circuits noir:build && \
  yarn tsc -b

ENTRYPOINT ["yarn"]
