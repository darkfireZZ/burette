#!/bin/sh

set -e

add_darwin && echo
add_faust && echo
echo

! burette remove abcdef
