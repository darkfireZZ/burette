#!/bin/sh

set -e

burette list

burette add $TEST_DOCS/darwin.epub << EOF
On the Origin of Species By Means of Natural Selection
yeS
Charles Darwin
N
no
y
10.5962/bhl.title.59991
EOF
echo
burette list

! burette add $TEST_DOCS/darwin.epub << EOF
On the Origin of Species By Means of Natural Selection
Yes
Charles Darwin
no
No
Y
10.5962/bhl.title.59991
EOF
echo
burette list
