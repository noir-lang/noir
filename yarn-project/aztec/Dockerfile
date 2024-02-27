FROM aztecprotocol/yarn-project-prod AS yarn-project-prod
ENTRYPOINT ["node", "--no-warnings", "/usr/src/yarn-project/aztec/dest/bin/index.js"]
EXPOSE 8080

# The version has been updated in yarn-project-prod.
# Adding COMMIT_TAG here to rebuild versioned image.
ARG COMMIT_TAG=""