#!/bin/bash

# use libafb development version if any
export LD_LIBRARY_PATH="/usr/local/lib64:$LD_LIBRARY_PATH"
export PATH="/usr/local/lib64:$PATH"
clear

if ! test -f $CARGO_TARGET_DIR/debug/libafb_i2c.so; then
    echo "FATAL: missing libafb_i2c.so use: cargo build"
    exit 1
fi

# start binder with test config
afb-binder -vvv --config=afb-binding/etc/binding-i2c.json
