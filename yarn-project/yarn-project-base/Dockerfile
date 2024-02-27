# This base Dockerfile is for:
#  - Caching the workspace dependencies.
#  - Running workspace checks (package.json inheritance, tsconfig.json project references).
#  - Bringing in any upstream resources such as L1 contract artifacts and bb.js.
#  - Performing any code generation that doesn't require the workspace code to do so (generate L1 artifacts).
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
#  - WORKDIR into the relevant project.
#  - Do any project specific build work.
#  - Do a `yarn workspaces focus --production` to install production dependencies from the global.
#  - Erase the local cache with a `yarn cache clean`.
#  - Destroy the yarn-project /src folders to save additional space.
#  - The above can be done with:
#      RUN yarn workspaces focus --production && yarn cache clean && rm -rf ../**/src
#  - Create final slim image by copying needed dirs into a fresh image.
#
FROM aztecprotocol/bb.js as bb.js
FROM aztecprotocol/noir-packages as noir-packages

FROM node:18.19.0
RUN apt update && apt install -y jq curl perl && rm -rf /var/lib/apt/lists/* && apt-get clean

# Copy in portalled packages.
COPY --from=bb.js /usr/src/barretenberg/ts /usr/src/barretenberg/ts
COPY --from=noir-packages /usr/src/noir/packages /usr/src/noir/packages

# We install a symlink to yarn-project's node_modules at a location that all portalled packages can find as they
# walk up the tree as part of module resolution. The supposedly idiomatic way of supporting module resolution
# correctly for portalled packages, is to use --preserve-symlinks when running node.
# This does kind of work, but jest doesn't honor it correctly, so this seems like a neat workaround.
# Also, --preserve-symlinks causes duplication of portalled instances such as bb.js, and breaks the singleton logic
# by initialising the module more than once. So at present I don't see a viable alternative.
RUN ln -s /usr/src/yarn-project/node_modules /usr/src/node_modules

WORKDIR /usr/src/yarn-project
# The dockerignore file ensures the context only contains package.json and tsconfig.json files.
COPY . .

# List all included files and hash for debugging.
RUN echo "Context files: " && find . -type f | sort && \
  echo -n "Context hash: " && find . -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum

# Install packages and rebuild the global cache with hard links.
# TODO: Puppeteer is adding ~300MB to this image due to chrome download (part of e2e).
#       Switch to using puppeteer-core then it won't download chrome. For now just erase.
RUN yarn --immutable && rm -rf /root/.cache/puppeteer && /bin/bash -c '\
  rm -rf /root/.yarn/berry/cache/* && \
  cd .yarn/cache && \
  for F in *; do \
  [[ $F =~ (.*-) ]] && ln $F /root/.yarn/berry/cache/${BASH_REMATCH[1]}8.zip; \
  done'

# Check package.json inheritance and tsconfig project references.
RUN yarn prepare:check