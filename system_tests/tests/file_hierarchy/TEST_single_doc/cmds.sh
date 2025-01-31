#!/bin/sh

set -e

LIBRARY_PATH=library

burette --library="$LIBRARY_PATH" new
add_faust && echo
tree "$LIBRARY_PATH"

echo
echo 'index.json:'
cat "$LIBRARY_PATH"/index.json
