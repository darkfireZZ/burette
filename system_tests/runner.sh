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

# Find the root directory of the project
GIT_DIR=$(git -C "$TEST_DIR" rev-parse --show-toplevel 2>/dev/null)

echo "Building project:" >&2
cargo build || exit 1

# Overwrite the PATH to include the version of the binary we just built
export PATH="$GIT_DIR/target/debug:$PATH"

# We want to isolate the tests from the user's home directory
#
# We use a separate variable to store the temporary directory to prevent
# anyone working on this script from accidentally deleting their entire home
# directory.
TMP_DIR=$(mktemp -d)
export HOME="$TMP_DIR"

SYS_TESTS="$TEST_DIR/tests"

export TEST_DOCS="$TEST_DIR/test_docs"

cd $SYS_TESTS

TESTS=$(find . -name 'TEST_*' -type d)

for sys_test in $TESTS; do
    echo -n "Running $sys_test... " >&2
    burette new > ./$sys_test/stdout 2> ./$sys_test/stderr && {
        ./$sys_test/cmds.sh > ./$sys_test/stdout 2> ./$sys_test/stderr
    }
    if [ $? -eq 0 ]; then
        echo -e " \033[0;32mpassed\033[0m" >&2
    else
        echo -e " \033[0;31mfailed\033[0m" >&2
        FAILED=1
    fi
    rm -r $TMP_DIR || exit 1
done

git diff --exit-code $SYS_TESTS
if [ $? -ne 0 ]; then
    echo "Unexpected test output" >&2
    FAILED=1
fi

if [ $FAILED -eq 0 ]; then
    echo -e "\033[0;32mAll tests passed\033[0m" >&2
    exit 0
else
    echo -e "\033[0;31mSome tests failed\033[0m" >&2
    exit 1
fi
