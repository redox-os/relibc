#!/bin/bash
set -ex

./fmt.sh -- --check
if [ -z "$TARGET" ]
then
    make all
    make test
else
    make libs
fi
