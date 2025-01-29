#!/bin/sh

set -e

add_darwin && echo
add_moby_dick && echo
add_faust && echo
add_var_chrom && echo
echo
burette list && echo

burette remove "$HASH_DARWIN" "$HASH_MOBY_DICK"
echo
burette list
