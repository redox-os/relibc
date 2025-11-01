#!/usr/bin/env bash

cargo fmt --package relibc --package crt0 --package redox-rt "$@"
