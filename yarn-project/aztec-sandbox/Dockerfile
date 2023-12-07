FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-prod AS yarn-project-prod

# The version has been updated in yarn-project-prod.
# Adding COMMIT_TAG here to rebuild versioned image.
ARG COMMIT_TAG=""

ENTRYPOINT ["node", "--no-warnings", "/usr/src/yarn-project/aztec-sandbox/dest/bin/index.js"]
EXPOSE 8079 8080
