#!/bin/sh

set -e

! burette add $TEST_DOCS/markdown_file.md << EOF
This is a markdown file
n
n
n
EOF

echo
burette list
