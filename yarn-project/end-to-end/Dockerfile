FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project AS builder

# Build aztec.js web.
WORKDIR /usr/src/yarn-project/aztec.js
RUN yarn build:web

WORKDIR /usr/src/yarn-project/end-to-end
# Productionify. See comment in yarn-project-base/Dockerfile.
RUN yarn cache clean && yarn workspaces focus --production

# Create final, minimal size image.
# TODO: Not very minimal as chromium adds about 500MB of bloat :/ Separate or install at test runtime?
FROM node:18-alpine
RUN apk update && apk add --no-cache \
  bash \
  chromium \
  chromium-chromedriver \
  nss \
  freetype \
  harfbuzz \
  ca-certificates \
  ttf-freefont

ENV CHROME_BIN="/usr/bin/chromium-browser" PUPPETEER_SKIP_CHROMIUM_DOWNLOAD="true"
COPY --from=builder /usr/src /usr/src

RUN rm /usr/src/yarn-project/end-to-end/src/web/aztec3-circuits.wasm
RUN rm /usr/src/yarn-project/end-to-end/src/web/main.js
COPY --from=builder /usr/src/circuits/cpp/build-wasm/bin/aztec3-circuits.wasm /usr/src/yarn-project/end-to-end/src/web/aztec3-circuits.wasm
COPY --from=builder /usr/src/yarn-project/aztec.js/dest/main.js /usr/src/yarn-project/end-to-end/src/web/main.js

WORKDIR /usr/src/yarn-project/end-to-end

ENTRYPOINT ["yarn", "test"]
