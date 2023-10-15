#!/bin/bash

# Get local IP address
local_ip=$(hostname -I | awk '{print $1}')

echo "Starting Rust server on $local_ip"
cd $PWD && ../target/release/chord $1 $2 &

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
	ssh $node "cd $PWD && ../target/release/chord $1 $2" &
done < <(head -n $1 ip.txt)

wait
