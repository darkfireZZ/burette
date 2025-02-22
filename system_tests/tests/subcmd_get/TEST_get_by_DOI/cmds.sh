#!/bin/sh

set -e

add_darwin && echo
add_moby_dick && echo
add_faust && echo
add_var_chrom && echo
echo

burette get '10.5962/bhl.title.59991'
ls
