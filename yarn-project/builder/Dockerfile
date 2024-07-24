FROM aztecprotocol/yarn-project AS yarn-project
ENTRYPOINT ["node", "--no-warnings", "/usr/src/yarn-project/builder/dest/bin/cli.js"]

# The version has been updated in yarn-project.
# Adding COMMIT_TAG here to rebuild versioned image.
ARG COMMIT_TAG=""
