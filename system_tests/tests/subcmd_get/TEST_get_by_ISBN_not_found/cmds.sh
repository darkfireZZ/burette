#!/bin/sh

set -e

add_darwin && echo
add_faust && echo
add_var_chrom && echo
echo

# ISBN of Moby Dick, this is not in the library and hence won't be found
! burette get '978-0198853695'
# ls should show no files
ls
