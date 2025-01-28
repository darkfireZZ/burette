#!/bin/sh

burette add $TEST_DOCS/markdown_file.md
if [ $? -eq 0 ]; then
    exit 1
else
    exit 0
fi
