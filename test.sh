#!/bin/bash
set -ex

make

make -C tests clean
make -C tests run
