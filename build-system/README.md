# Build System

The Aztec build system is agnostic to its underlying platform, but currently our builds run in Circle CI. There were several requirements to be met in it's design.

## Requirements

- Monorepo support (or at least, multiple projects within one repoistory).
- Builds docker containers for simple deployments.
- Docker layer caching support to minimise rebuild times.
- Don't rebuild projects that haven't changed as part of a commit (analyse diffs between commits).
- Allow fine or coarse grained control, of which file changes within a project, trigger a rebuild.
- Stateless (apart from the source repository itself, and the target container registry).
- Enable building on powerful (up to 64 core) EC2 spot instances. They're extremely cheap and powerful relative to Circle CI offerings.
- Easy to follow build graph on Circle CI.
- Deploy updated services only on a fully successful build of entire project.
- No vendor lock-in (don't use vendor specific features).

## Overview

We will assume Circle CI is the orchestration platform

There are scripts that are called from the `.circleci/config.yml` that could be fairly easily run elsewhere if needed. They are located in the `scripts` folder, and are added to `PATH` so they can be called from project directories. The actual building of the services and libraries are all done with Dockerfiles.

There are two ECR (elastic container repository) instances used in two regions (`eu-west2` and `us-east2`). As containers are built, the results are stored in `us-east2` (deemed to be generally close to Circle CI) and these are considered to be caches that can be reused in subsequent builds. In the event of a deploy, the containers are published in `eu-west2` where all infrastructure is currently hosted. These are considered our live production builds.

We do not use Circle CI's "docker layer caching" feature, because:

- There is no guarantee the cache will be available between workflow steps or builds.
- There is not one single cache, but multiple caches which are randomly attached to your job.

For this reason it's undeterministic in terms of state or performance, and is thus impossible to use it for anything useful.

## Important Concepts

We avoid using any Circle CI specific features. They are very general purpose, and are thus often flawed. Also, we don't want vendor lock-in as Circle CI has caused us multiple problems in the past. We only use Circle CI to orchestrate the build sequence. We could relatively easily shift this orchestration to another vendor, or custom internal build service.

The build system leverages image names and tags in the docker image registry to keep track of it's historical success or failure in terms of builds, tests, and deployments. It's otherwise stateless, meaning it only needs a container registry to track state.

We work in terms of _commits_ and not branches. Branches are a higher level concept that are ignored. Given a commit hash, there is a linear history of commits we scan and compare to the docker registry to determine what's changed, and thus what needs to be rebuilt.

There is a `build_mainfest.json` that desribes various settings for each project (dependencies, rebuild patterns, etc). The dependencies as listed in the build manifest represent the graph such that if project A changes, all projects that depend on A will also be rebuilt. This likely closely mirrors the workflow graph as defined in Circle CI's `config.yml`.

A rebuild pattern is a regular expression that is matched against a list of changed files. We use pretty broad regular expressions that trigger rebuilds if _any_ file in a project changes, but you can be more fine-grained, e.g. not triggering rebuilds if you change something inconsequential.

## Usage

Add the build system into your repository as a submodule located at `/build-system`. Circle CI expects a `.circleci/config.yml` file from which you can leverage the build scripts. After checking out your repository code, initialise this submodule e.g.

```
git submodule update --init build-system
```

At the start of each job, it's necessary to setup the build environment e.g.

```
./build-system/scripts/setup_env "$CIRCLE_SHA1" "$CIRCLE_TAG" "$CIRCLE_JOB" "$CIRCLE_REPOSITORY_URL" "$CIRCLE_BRANCH"
```

Once called all scripts are available directly via `PATH` update, and various other env vars expected by scripts are set. You'll want to `source` the above script if you intend to use the build system within the calling shell.

Jobs will usually leverage one of the following scripts. View the scripts themselves for further documentation:

- `build`
- `deploy`
- `deploy_global`
- `cond_spot_run_build`
- `cond_spot_run_tests`

There are more fine grained scripts that maybe used in some cases such as:

- `deploy_ecr`
- `deploy_terraform`
- `deploy_npm`
- `deploy_s3`
- `deploy_dockerhub`
