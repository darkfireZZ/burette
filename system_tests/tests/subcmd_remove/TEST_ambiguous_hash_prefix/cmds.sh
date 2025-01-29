#!/bin/sh

set -e

add_darwin && echo
add_moby_dick && echo
add_faust && echo
add_var_chrom && echo
echo
burette list && echo

! burette remove 2 "$HASH_DARWIN"
echo
burette list
