#!/bin/sh

set -e

burette list

burette add $TEST_DOCS/moby_dick_1.epub << EOF
Moby-Dick
yes
Herman Melville
no
yes
978-0198853695
yes
9788417517212
n
nO
EOF
echo
burette list

! burette add $TEST_DOCS/moby_dick_2.epub << EOF
Moby-Dick
YES
Herman Melville
NO
YES
978-1092312035
yEs
9780198853695
N
No
EOF
echo
burette list
