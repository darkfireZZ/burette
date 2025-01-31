#!/bin/sh

set -e

add_darwin && echo
add_var_chrom && echo
add_faust && echo
add_moby_dick && echo
echo
burette list

rm $HOME/.book-store/documents/$HASH_VAR_CHROM

echo
! burette validate
