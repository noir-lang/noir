#!/bin/bash

# Change the scope to @kevaundray so I can publish to my account for now
bash ./change-scope.sh

# update the versions in the workspace package.json files
# so that they become version-{branch-name}-{commit-hash}
bash ./update-version.sh

# publish and patch the workspace dependencies
# TODO: can move patching the workspace dependencies to the update-version script
bash ./publish-script.sh