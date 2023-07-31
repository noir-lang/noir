#!/bin/bash
set -e

# Run clang-tidy on all C++ source files
#
# CMake must be configured for clang15 before running this script:
#     cmake --preset clang15
#
# Run examples (from circuits/cpp)
# ./scripts/tidy.sh
# or
# ./scripts/tidy.sh fix

###############################################################################
# ARGS
###############################################################################
# FIX: <empty> or fix (MUST BE LOWERCASE)
#      if 'fix', apply fixes to actual source file
#      else, print warnings
FIX=$1
# END ARGS
###############################################################################

# CMake build dir which should contain `compile_commands.json`
BUILD_DIR=build
echo "*************************************************************************"
if [ "$FIX" == "fix" ]; then
  echo "Tidy all source files - fix and format the files themselves"
  FIX_OPT="-fix -format"
else
  EXPORT_DIR=$BUILD_DIR/tidy-fixes
  # Clean old tidy fixes
  rm -f $EXPORT_DIR/*
  mkdir -p $EXPORT_DIR
  YAML_FILE=$EXPORT_DIR/tidy-all.yaml

  # run tidy on each source file and export fixes to yaml files
  echo "Checking tidy on all source files and exporting fixes to directory: $EXPORT_DIR"
  FIX_OPT="-export-fixes $YAML_FILE"
  echo "To apply these fixes later, run the following from circuits/cpp:"
  echo "    clang-apply-replacements --format --style file $EXPORT_DIR"
fi


# find all C++ source files to tidy up
SOURCES=$(\
  find src/aztec3/ \
     -name *.cpp \
  -o -name *.hpp \
  -o -name *.cxx \
  -o -name *.hxx \
  -o -name *.tpp \
  -o -name *.cc  \
  -o -name *.hh  \
  -o -name *.c   \
  -o -name *.h   \
  | LC_ALL=C sort \
)

# MD5 of all source files before running clang-tidy
BEFORE_MD5=$(
  for src in ${SOURCES[@]}; do
    md5sum $src
  done | md5sum)
echo "Before running clang-tidy, MD5 of all C++ files was: $BEFORE_MD5"
echo "*************************************************************************"

# Need run-clang-tidy version 15, but it doesn't have a --version flag
RUN_TIDY=$(which run-clang-tidy-15 || which run-clang-tidy || which run-clang-tidy-mp-15)
# tidy all sources
$RUN_TIDY -p $BUILD_DIR $SOURCES $FIX_OPT -use-color || {\
  echo "Errors encountered when running clang-tidy!" &&
  echo "Check the output above before trying again." && \
  exit 1;
}

echo "*************************************************************************"
if [ "$FIX" == "fix" ]; then
  # MD5 of all source files after running clang-tidy
  AFTER_MD5=$(
    for src in ${SOURCES[@]}; do
      md5sum $src
    done | md5sum)
  echo "AFTER running clang-tidy, MD5 of all C++ files was: $AFTER_MD5"

  echo "$BEFORE_MD5 ?= $AFTER_MD5"
  if [ "$BEFORE_MD5" == "$AFTER_MD5" ]; then
    echo "No tidying necessary!"
    exit 0
  else
    echo "WARNING: Some tidying was necessary!"
    echo "If you are seeing this in CI, run the following"
    echo "in circuits/cpp to tidy up the C++:"
    echo "    ./scripts/tidy.sh fix"
    exit 1
  fi
else
  echo "Reminder!"
  echo "To apply these fixes later, run the following from circuits/cpp:"
  echo "    clang-apply-replacements --format --style file $EXPORT_DIR"
fi
echo "*************************************************************************"
