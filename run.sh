#!/bin/bash

# Check if the user supplied an argument
if [ "$#" -ne 1 ]; then
	echo "Usage: $0 <nodes>"
	exit 1
fi

# The user supplied argument
PARAMETER=$1

# SSH into c7-1 and execute the start_server.sh script with the given parameter
ssh c7-1 "cd $PWD; ./src/start_server.sh $PARAMETER 61021"

echo "Sleeping until servers are ready..."
sleep 15
echo "Servers are ready!"
# Once the SSH command completes, run the Python script on the local machine
/share/python3115/bin/python3 connect.py
