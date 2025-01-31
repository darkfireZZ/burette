#!/bin/sh

set -e

add_darwin && echo
add_var_chrom && echo
add_faust && echo
add_moby_dick && echo
echo
burette list

touch some_file
ln -s some_file $HOME/.book-store/documents/unexpected_symlink

echo
! burette validate
