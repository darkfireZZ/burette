#!/bin/sh

set -e

burette add $TEST_DOCS/var_chrom.pdf << EOF
Variations Chromatiques de concert
y
Georges Bizet
n
n
n
EOF

burette list
