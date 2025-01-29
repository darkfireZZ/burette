#!/bin/sh

set -e

burette --library new_lib new
ls new_lib

echo
echo 'index.json:'
cat new_lib/index.json
echo
echo 'burette_version:'
cat new_lib/burette_version
