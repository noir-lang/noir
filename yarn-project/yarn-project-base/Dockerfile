# This base Dockerfile is for:
#  - Caching the workspace dependencies.
#  - Running workspace checks (package.json inheritence, tsconfig.json project references).
#  - Bringing in any upstream artefacts such as circuits.wasm and L1 contracts.
#  - Performing any code generation that doesn't require the workspace code to do so (generate L1 artefacts).
#
# When installing workspace dependencies, there are issues with doing this naively:
#  - We only ever want to re-download and install workspace dependencies in the event that they change.
#    If a developer only changes some code, we want this step of the build to become a noop and to reuse existing image.
#    Dockerfile.dockerignore is tailored to specifically only bring in files needed to install the dependencies.
#    NOTE: For this dockerignore file to be used, you MUST be using BUILDKIT in more recent versions of Docker.
#          Best to make sure you're on docker >= 24. On mainframe run `restart_docker`, it should auto-upgrade.
#  - We want to disable yarn from accessing the net after initial package download as it prevents a class of build bugs.
#  - We want to prune dev dependencies as this can significantly reduce the final container size even further.
#    Yarn devs won't provide the ability to prune dev dependencies from the local project cache:
#      https://github.com/yarnpkg/berry/issues/1789
#    This means we need a global cache, so we can clean the cache and reinstall prod modules without re-downloading.
#  - The default global cache and local cache are simply copies, not hardlinks, thus doubling the storage of modules.
#    To work around, we will construct a global cache from the local cache using hard links (requires a hacky rename).
#    We do this in the same layer the original file is created, otherwise overlayfs creates a copy anyway.
#    At time of writing this shaves off around 150MB. Not a big deal but a harmless trick for now.
#
# So, here in the base image we yarn install.
#  - /root/.yarn/berry/cache (global cache) is populated with zipped packages.
#  - /usr/src/yarn-project/.yarn/cache (project local cache) is populated with copy of zipped packages.
#  - Packages are unzipped into various node_modules folders.
#  - We then erase the global cache, and recreate each file as a hard link, reducing the zipped package storage by half.
#
# That's all we want to do here r.e. dependency installation. In yarn-project we will:
#  - Copy in the rest of the workspace files.
#  - Perform any code generation steps that require the workspace code (generate L2 contract wrappers).
#  - Build all the TypeScript.
#
# When we build a downstream docker image, we:
#  - FROM yarn-project as builder, to get the entire built project.
#  - Do any project specific work.
#  - Erase the local cache with a `yarn cache clean`. Files will remain in global cache.
#  - Do a `yarn workspaces focus --production` to install production dependencies from the global to local cache.
#  - The above can be done with:
#      RUN yarn cache clean && yarn workspaces focus --production
#  - Create final slim image by copying /usr/src into a fresh image.
#
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/circuits-wasm-linux-clang as circuits
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/l1-contracts as contracts

FROM node:18-alpine
RUN apk update && apk add --no-cache bash jq
WORKDIR /usr/src/yarn-project

# The dockerignore file ensures the context only contains package.json and tsconfig.json files.
COPY . .

# Install packages and rebuild the global cache with hard links.
# TODO: Puppeteer is adding ~300MB to this image due to chrome download (part of e2e).
#       Switch to using puppeteer-core then it won't download chrome. For now just erase.
RUN yarn --immutable && rm -rf /root/.cache/puppeteer && /bin/bash -c '\
  rm -rf /root/.yarn/berry/cache/* && \
  cd .yarn/cache && \
  for F in *; do \
    [[ $F =~ (.*-) ]] && ln $F /root/.yarn/berry/cache/${BASH_REMATCH[1]}8.zip; \
  done'

# If everything's worked properly, we should no longer need access to the network.
RUN echo "enableNetwork: false" >> .yarnrc.yml

# Check package.json inheritence and tsconfig project references.
RUN yarn prepare:check

# Bring in circuits wasms.
COPY --from=circuits /usr/src/circuits/cpp/build-wasm/bin /usr/src/circuits/cpp/build-wasm/bin

# Generate L1 contract TypeScript artifacts.
COPY --from=contracts /usr/src/l1-contracts/out /usr/src/l1-contracts/out
RUN cd l1-artifacts && ./scripts/generate-artifacts.sh
