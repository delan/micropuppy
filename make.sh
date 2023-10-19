#!/bin/sh
# usage: ./make.sh <debug|release> <make args ...>
set -eu

mode=$1; shift
case "$mode" in
    (debug)
        set -- CARGO_PROFILE_DIR=debug CARGO_PROFILE_FLAG= "$@"
        ;;
    (release)
        set -- CARGO_PROFILE_DIR=release CARGO_PROFILE_FLAG=--release "$@"
        ;;
    (*)
        >&2 echo 'fatal: $1 should be debug or release'
esac

exec make "$@"
