#!/bin/sh

burette --library . add $TEST_DOCS/var_chrom.pdf

if [ $? -eq 0 ]; then
    exit 1
else
    exit 0
fi
