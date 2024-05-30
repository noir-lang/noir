FROM aztecprotocol/yarn-project AS yarn-project
# ENV vars for using native ACVM simulation
ENV BB_BINARY_PATH=/usr/src/barretenberg/cpp/build/bin/bb
ENV BB_WORKING_DIRECTORY=/usr/src/yarn-project/bb
ENV ACVM_BINARY_PATH=/usr/src/noir/noir-repo/target/release/acvm
ENV ACVM_WORKING_DIRECTORY=/usr/src/yarn-project/acvm
RUN mkdir -p $BB_WORKING_DIRECTORY $ACVM_WORKING_DIRECTORY
ENTRYPOINT ["node", "--no-warnings", "/usr/src/yarn-project/aztec/dest/bin/index.js"]
EXPOSE 8080

# The version has been updated in yarn-project.
# Adding COMMIT_TAG here to rebuild versioned image.
ARG COMMIT_TAG=""
