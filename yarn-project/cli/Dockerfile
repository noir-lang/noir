FROM aztecprotocol/yarn-project AS yarn-project
ENTRYPOINT ["node", "--no-warnings", "/usr/src/yarn-project/cli/dest/bin/index.js"]

# The version has been updated in yarn-project.
# Adding COMMIT_TAG here to rebuild versioned image.
ARG COMMIT_TAG=""

RUN mkdir /cache && chmod 777 /cache
ENV XDG_CACHE_HOME /cache
VOLUME "/cache"
