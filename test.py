import requests
import time
import csv
# Define the server URL
server_url = "http://172.21.21.175:65000/storage/2"



num_requests = 100  # Adjust as needed


# Functi n to perform a PUT request
def perform_put_request():
    global put_count
    response = requests.put(server_url, data='data.txt')

# Function to perform a GET request
def perform_get_request():
    global get_count
    response = requests.get(server_url)
for i in range(10):
    # Measure the throughput
    start_time = time.time()

    for i in range(num_requests):
        perform_get_request()

    end_time = time.time()

    # Calculate throughput
    throughput = num_requests / (end_time - start_time)
    with open("results.csv", mode="a", newline="") as file:
        writer = csv.writer(file)
        writer.writerow([4, throughput, "get"])

    print(f"Throughput (requests per second): {throughput:.2f}")
