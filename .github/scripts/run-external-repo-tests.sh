#!/usr/bin/env bash
set -eu

NARGO=${NARGO:-nargo}

# Make output paths absolute to ensure they don't get written inside of temp directory.
OUTPUT_FILE=$(realpath -m $OUTPUT_FILE)
BENCHMARK_FILE=$(realpath -m $BENCHMARK_FILE)

mkdir -p $(dirname $OUTPUT_FILE)
mkdir -p $(dirname $BENCHMARK_FILE)

if [ -z "${CI:-}" ]; then
    # Sadly we cannot use depth=1 clones here as we need to be able to checkout
    # commit hashes as well as branches/releases
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT
    git clone $REPO $TMP_DIR
    git -C $TMP_DIR -c advice.detachedHead=false checkout $TAG
fi

REPO_DIR=${REPO_DIR:-$TMP_DIR}

cd $REPO_DIR/$PROJECT_PATH

set +e
sed -i '/^compiler_version/d' {Nargo.toml,./**/Nargo.toml}
set -e

BEFORE=$SECONDS
$NARGO test --silence-warnings --skip-brillig-constraints-check --pedantic-solving --format json $NARGO_ARGS | tee $OUTPUT_FILE
TIME=$(($SECONDS-$BEFORE))

if [ ! -s $OUTPUT_FILE ]; then
# The file is empty so we delete it to signal that `nargo test` failed before it could run any tests
rm -f $OUTPUT_FILE
fi

jq --null-input "[{ name: \"$NAME\", value: (\"$TIME\" | tonumber), unit: \"s\" }]" > $BENCHMARK_FILE
