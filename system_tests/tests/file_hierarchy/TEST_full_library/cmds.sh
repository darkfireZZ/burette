#!/bin/sh

set -e

LIBRARY_PATH=full_library

burette --library="$LIBRARY_PATH" new
add_faust && echo
add_var_chrom && echo
add_moby_dick && echo
add_darwin && echo
echo
tree "$LIBRARY_PATH"

echo
echo 'index.json:'
cat "$LIBRARY_PATH"/index.json
