#!/bin/sh

set -e

add_darwin && echo
add_moby_dick && echo
echo

burette get $HASH_DARWIN -o custom_name.epub
burette get $HASH_DARWIN

sha256sum *
