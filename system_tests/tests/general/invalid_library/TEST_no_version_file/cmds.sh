#!/bin/sh

set -e

TMPDIR=$(mktemp -d)
# We enter the temporary directory to ensure that its path does not appear in
# the output of the tests.
cd $TMPDIR

LIBRARY=./library

burette --library $LIBRARY new
rm $LIBRARY/burette_version

! burette --library $LIBRARY add $TEST_DOCS/var_chrom.pdf

rm -rf $LIBRARY
rm -d $TMPDIR
