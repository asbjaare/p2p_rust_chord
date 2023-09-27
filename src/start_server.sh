#!/bin/bash

# Get local IP address
local_ip=$(hostname -I | awk '{print $1}')

echo "Starting Rust server on $local_ip"
cargo run &

# Read IP addresses from ip.txt file
while read -r line || [[ -n "$line" ]]; do
    # Extract node name and IP address from line
    node=$(echo $line | awk '{print $1}')
    ip=$(echo $line | awk '{print $3}')

    # Skip local node
    if [ "$ip" = "$local_ip" ]; then
        echo "Skipping local node $node"
        continue
    fi

    echo "Starting Rust server on $node"
    ssh $node "cd /mnt/users/ita022/inf-3200/p2p_rust_chord/src && cargo run" &
done < ip.txt

wait