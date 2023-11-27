FROM node:18-alpine AS builder

RUN apk update && apk add --no-cache bash jq curl

COPY ./end-to-end /usr/src/end-to-end
COPY ./canary /usr/src/canary

# Setup & Build end-to-end package
WORKDIR /usr/src/end-to-end
# Extract canary @aztec dependencies
RUN ../canary/scripts/extract_packages.sh ../canary/package.json > ./target_pkgs.txt
# Update end-to-end dependencies with target version
RUN ./scripts/setup_canary.sh ./target_pkgs.txt package.json
RUN rm ./target_pkgs.txt
# yarn install & build to apply changes (canary packages)
RUN yarn && yarn build

# Install & build canary packages
WORKDIR /usr/src/canary
RUN ./scripts/update_packages.sh canary ../end-to-end/
RUN yarn && yarn build

FROM node:18-alpine
RUN apk update && apk add --no-cache udev ttf-freefont chromium bash
ENV CHROME_BIN="/usr/bin/chromium-browser" PUPPETEER_SKIP_CHROMIUM_DOWNLOAD="true"

COPY --from=builder /usr/src/end-to-end /usr/src/end-to-end
COPY --from=builder /usr/src/canary /usr/src/canary

WORKDIR /usr/src/canary
# Copy browser script
RUN cp ../end-to-end/scripts/start_e2e_ci_browser.sh ./scripts/start_e2e_ci_browser.sh
RUN chmod +x scripts/start_e2e_ci_browser.sh

RUN cp ./node_modules/@aztec/aztec.js/dest/main.js src/web/

ENTRYPOINT ["yarn", "test"]
