# yarn-project-base

## Why?

If you want to rebuild a docker container for a project in the workspace, you don't want to have to be waiting
to download the entire set of workspace dependencies just because you changed a line of code. The way docker caches
and reuses layers is very powerful. We build this base image in order to:

1. Encapsulate the downloading of all workspace dependencies.
1. Check our package.json files have inherited from the common package.json.
1. Check out tsconfig project references are all correct.
1. Generate L1 contract ABIs.

The root project Dockerfile `yarn-project` then:

1. Generates Noir contract ABIs.
1. Builds the entire project.
1. Checks all formatting is correct.
1. Runs all workspace unit tests.

Downstream projects are then just about containerising what's needed to produce executable containers for e2e testing or
deployments.

## Do we care about docker layer caching, when build-system rebuild patterns only trigger on yarn.lock changes?

Enough. When building the containers locally for development or debugging purposes, you can't use the content hash
to skip parts of the build, as content hashes require everything to have been committed to git. This is usually
is not the case during development.
