#!/bin/sh

set -e

cmd() {
    burette add $TEST_DOCS/var_chrom.pdf << EOF
Variations Chromatiques de concert
y
Georges Bizet
n
n
n
EOF
}

cmd
echo
! cmd
echo

burette list
