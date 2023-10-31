#!/bin/bash

nprime="172.21.21.175:60021"
# start_time=$(date +%s)
i=0

# Skip the first line of the file
tail -n +2 ip.txt | head -n 49 | while read line
do
    ip_addr=$(echo $line | awk '{print $3}')
    i=$((i+1))
    echo $ip_addr
    echo $i

    response=$(curl -s -o /dev/null -w "%{http_code}" -X POST "http://$ip_addr:60021/join?nprime=$nprime")
    nprime="$ip_addr:60021"
    echo $response
    # sleep 0.1
done

# end_time=$(date +%s)
# total_time=$((end_time-start_time))
# echo $total_time