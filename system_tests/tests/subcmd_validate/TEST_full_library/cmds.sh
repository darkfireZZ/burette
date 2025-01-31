#!/bin/sh

set -e

add_darwin && echo
add_var_chrom && echo
add_faust && echo
add_moby_dick && echo
echo
burette list
echo
burette validate
