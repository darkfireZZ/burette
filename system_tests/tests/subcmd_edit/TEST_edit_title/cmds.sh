#!/bin/sh

set -e

# Add a couple of documents
add_moby_dick && echo
add_darwin && echo
burette list && echo

# Edit the document
burette edit "$HASH_DARWIN" title << EOF
Some new title for Darwin's old bookkk
EOF
echo
burette list && echo

# Change it back
burette edit "$HASH_DARWIN" title << EOF
On the Origin of Species blabla
EOF
echo
burette list && echo
