#!/bin/sh

# The test runner already generates a library for the test.
# Calling burette new will fail because the library already exists.
! burette new
