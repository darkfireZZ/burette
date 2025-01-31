#!/bin/bash

# This script runs all the system tests in the tests directory.
#
# The script builds the project and then runs each test in the tests directory.
# If any test fails, the script will print an error message and return a non-zero
# exit code.

echo -e "\033[1mRunning system tests\033[0m" >&2

FAILED=0

# Path to the directory containing this script
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $TEST_DIR

echo "Building project:" >&2
cargo build || exit 1

# We want to isolate the tests from the user's home directory
#
# We use a separate variable to store the temporary directory to prevent
# anyone working on this script from accidentally deleting their entire home
# directory.
TMP_DIR=$(mktemp -d)
export HOME="$TMP_DIR"

export TEST_DOCS="$TEST_DIR/test_docs/"
export LIBRARY_PATH="$HOME/.book-store/"

# Load the helper functions
# - `add_darwin`
# - `add_faust`
# - `add_moby_dick`
. helpers.sh

# Find the root directory of the project
GIT_DIR=$(git -C "$TEST_DIR" rev-parse --show-toplevel 2>/dev/null)

# Overwrite the PATH to include the version of the binary we just built
export PATH="$GIT_DIR/target/debug:$PATH"

SYS_TESTS="$TEST_DIR/tests/"

cd $SYS_TESTS

TESTS=$(find . -name 'TEST_*' -type d)

for sys_test_rel in $TESTS; do
    sys_test="$SYS_TESTS/$sys_test_rel"
    mkdir -p $TMP_DIR
    echo -n "Running ${sys_test_rel#./}... " >&2
    (cd $HOME && burette new && $sys_test/cmds.sh) \
        < /dev/null > $sys_test_rel/stdout 2> $sys_test_rel/stderr \
        && git diff --quiet $sys_test_rel
    if [ $? -eq 0 ]; then
        echo -e " \033[0;32mpassed\033[0m" >&2
    else
        echo -e " \033[0;31mfailed\033[0m" >&2
        FAILED=1
    fi
    rm -r $TMP_DIR || exit 1
done

if [ $FAILED -eq 0 ]; then
    echo -e "\033[0;32mAll tests passed\033[0m" >&2
else
    echo -e "\033[0;31mSome tests failed\033[0m" >&2
fi

exit $FAILED
