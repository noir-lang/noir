FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project AS builder
ARG COMMIT_TAG="."

COPY . .

# Setup & Build end-to-end package
WORKDIR /usr/src/yarn-project/end-to-end

# Extract canary @aztec dependencies
RUN ../canary/scripts/extract_packages.sh ../canary/package.json > ./target_pkgs.txt
# Update end-to-end dependencies with target version
RUN ./scripts/setup_canary.sh "$COMMIT_TAG" ./target_pkgs.txt
RUN rm ./target_pkgs.txt
RUN yarn && yarn build

# Build canary package
WORKDIR /usr/src/yarn-project/canary
RUN ./scripts/update_packages.sh "$COMMIT_TAG"
RUN yarn && yarn build

FROM node:18-alpine
RUN apk update && apk add --no-cache udev ttf-freefont chromium curl jq bash
ENV CHROME_BIN="/usr/bin/chromium-browser" PUPPETEER_SKIP_CHROMIUM_DOWNLOAD="true"

COPY --from=builder /usr/src/ /usr/src/
WORKDIR /usr/src/yarn-project/canary

RUN apk update && apk add --no-cache udev ttf-freefont chromium curl jq bash
ENV CHROME_BIN="/usr/bin/chromium-browser" PUPPETEER_SKIP_CHROMIUM_DOWNLOAD="true"

# Copy web artifacts for browser test
WORKDIR /usr/src/yarn-project/aztec.js
RUN yarn build:web
WORKDIR /usr/src/yarn-project/canary
RUN cp ../aztec.js/dest/main.js src/web/
RUN cp ../circuits.js/resources/aztec3-circuits.wasm src/web/
ENTRYPOINT ["yarn", "test"]
