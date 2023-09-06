FROM node:18-alpine
RUN apk update && apk add --no-cache jq bash

ARG COMMIT_TAG=""

#Build canary
WORKDIR /usr/src/
COPY ./canary .
RUN ./scripts/update_packages.sh $COMMIT_TAG
RUN yarn && yarn build

ENTRYPOINT ["yarn", "test"]