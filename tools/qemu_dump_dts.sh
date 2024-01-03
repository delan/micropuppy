#!/bin/sh
# usage: ./qemu_dump_dts.sh [qemu args]
# DTS decompiled from DTB is printed to stdout, and QEMU monitor output is printed to stderr.
DTB_PA=0x40000000
DTB_MAX_SIZE=0x400000 # 4MiB

dtb=`mktemp --suffix=.dtb`
trap "rm $dtb" EXIT

echo "pmemsave $DTB_PA $DTB_MAX_SIZE \"$dtb\"
quit" \
    `# need -serial vc to not conflict with monitor - does nothing with -nographic` \
    | qemu-system-aarch64 "$@" -nographic -serial vc -monitor stdio 1>&2 \
    || { echo "error: could not dump dts :("; exit 1; }

dtc -I dtb $dtb # prints to stdout
