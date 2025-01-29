#!/bin/sh

set -e

burette list && echo

add_moby_dick && echo
burette list && echo

add_darwin && echo
burette list && echo

add_faust && echo
burette list &&  echo

# Remove darwin again
burette remove 1904714f169d && echo
burette list && echo
