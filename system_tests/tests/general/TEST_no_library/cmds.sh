#!/bin/sh

burette -l ./non-existent-library add $TEST_DOCS/faust_teil_1.epub

if [ $? -eq 0 ]; then
    exit 1
else
    exit 0
fi
