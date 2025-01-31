#!/bin/sh

# `burette remove` allows removal of multiple documents at once.
# The documents are identified by a prefix of their hash.
# This test calls `remove` with prefixes that are nested.

set -e

LIBRARY_PATH=./library

burette -l $LIBRARY_PATH new

add_darwin && echo
add_moby_dick && echo
add_faust && echo
add_var_chrom && echo
echo
burette -l $LIBRARY_PATH list && echo

# $HASH_VAR_CHROM starts with 2576
! burette -l $LIBRARY_PATH remove '' 2 25 257 2576 $HASH_VAR_CHROM $HASH_DARWIN
echo
burette -l $LIBRARY_PATH list
