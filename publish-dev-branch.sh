#!/bin/bash

# Change the scope to @kevaundray so I can publish to my account for now
bash ./change-scope.sh

# Run yarn and yarn build so that the built artifacts are up to date;
# They should now reference the changed scope package names 
yarn && yarn build

# update the versions in the workspace package.json files
# so that they become version-{branch-name}-{commit-hash}
bash ./update-version.sh

# publish and patch the workspace dependencies
bash ./publish-script.sh