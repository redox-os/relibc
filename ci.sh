#!/bin/bash
set -ex

./fmt.sh -- --write-mode=diff
make
if [ -z "$TARGET" ]
then
    make test
fi
