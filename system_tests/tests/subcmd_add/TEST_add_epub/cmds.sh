#!/bin/sh

set -e

burette add $TEST_DOCS/faust_teil_1.epub  << EOF
Faust: Eine Tragödie [erster Teil]
y
Johann Wolfgang von Goethe
n
n
n
EOF

burette list
