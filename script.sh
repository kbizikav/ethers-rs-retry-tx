#!/bin/bash

for i in {1..10}
do
    echo "=== Test Run $i of 10 ==="
    cargo test -r approve_token -- --nocapture
    echo ""
done