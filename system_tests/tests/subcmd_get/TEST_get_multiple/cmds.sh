#!/bin/sh

set -e

add_darwin && echo
add_moby_dick && echo
add_faust && echo
echo
burette get "$HASH_DARWIN"
burette get "$HASH_MOBY_DICK"
burette get "$HASH_FAUST"
ls
