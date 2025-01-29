#!/bin/sh

set -e

add_darwin && echo
add_moby_dick && echo
add_faust && echo
echo

! burette get ''
