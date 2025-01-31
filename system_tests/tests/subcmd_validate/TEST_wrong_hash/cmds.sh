#!/bin/sh

set -e

add_darwin && echo
add_faust && echo
echo
burette list

mv $HOME/.book-store/documents/$HASH_DARWIN $HOME/.book-store/documents/$HASH_MOBY_DICK

echo
! burette validate
