#!/bin/sh

set -e

add_darwin && echo
add_moby_dick && echo
add_faust && echo
add_var_chrom && echo
echo

burette get '978-0198853695'
ls
