#!/bin/sh

set -e

add_darwin && echo
add_var_chrom && echo
add_faust && echo
add_moby_dick && echo
echo
burette list

rm "$LIBRARY_PATH"/burette_version

# We need to set the library path to a relative path so that the test output
# will not contain $HOME which may vary.
! burette -l ./.book-store validate
