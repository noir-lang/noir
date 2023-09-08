FROM node:18-alpine
RUN apk update && apk add --no-cache udev ttf-freefont chromium curl jq bash
ENV CHROME_BIN="/usr/bin/chromium-browser" PUPPETEER_SKIP_CHROMIUM_DOWNLOAD="true"

ARG COMMIT_TAG=""

#Build canary
WORKDIR /usr/src/
COPY ./canary .
RUN ./scripts/update_packages.sh $COMMIT_TAG
RUN yarn && yarn build

RUN cp node_modules/@aztec/aztec.js/dest/main.js src/web/
RUN cp node_modules/@aztec/circuits.js/resources/aztec3-circuits.wasm src/web/

ENTRYPOINT ["yarn", "test"]