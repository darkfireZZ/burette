#!/bin/sh

set -e

add_darwin && echo
add_faust && echo
echo

! burette get 2
