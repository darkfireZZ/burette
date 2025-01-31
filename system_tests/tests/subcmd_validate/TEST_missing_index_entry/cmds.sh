#!/bin/sh

set -e

add_darwin && echo
add_var_chrom && echo
add_faust && echo
add_moby_dick && echo
echo
burette list

cp $HOME/.book-store/documents/$HASH_MOBY_DICK .
echo
burette remove $HASH_MOBY_DICK
mv $HASH_MOBY_DICK $HOME/.book-store/documents/

! burette validate
