#!/bin/bash
set -ex

./fmt.sh -- --write-mode=diff
if [ -z "$TARGET" ]
then
    make all
    make test
else
    make libc
fi
