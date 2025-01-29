
# System Tests

This directory contains the system tests for `burette`.
These tests are run against the compiled binary and test the program as a whole.

## Running Tests

System tests are run using the custom runner [`runner.sh`](./runner.sh) or the makefile at the
project root (which calls the runner).

## Writing Tests

Tests are written in shell script.
A single test is a directory with a name that starts with `TEST_` and contains a `cmds.sh` file.
The `cmds.sh` file is the actual test script.
The test fails if the script exits with a non-zero status.
This means that most likely you will want to use `set -e` at the beginning of the script.

### Test Output

The runner also captures the output of the test script in the files `stdout` and `stderr` in the
directory of the test.
If the output differs from the previous run, the runner will print the diff and fail the test.

### Test Environment

- `burette` is compiled in debug mode and is prepended to `PATH`.
- Standard input is redirected from `/dev/null`.
- The `$HOME` directory is set to a temporary directory in which a fresh `burette` library is
  already initialized.
  This means that the tests can simply use `burette` commands without having to worry about
  creating and specifying the path to a library.
- The test documents from [test_docs](test_docs) are available in the directory stored in
  `$TEST_DOCS`.
- The helper functions `add_darwin`, `add_faust`, `add_moby_dick` and `add_var_chrom` are made
  available.
  These are shorthands for adding the corresponding test documents to the library.
- The environment variables `$HASH_DARWIN`, `$HASH_FAUST`, `$HASH_MOBY_DICK` and `$HASH_VAR_CHROM`
  contain the SHA-256 hash of the respective test document.
