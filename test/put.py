import requests
from time import sleep as wait
import random
import time

# key = input("what key:")
nprime = "172.21.21.175:55557"

def put_random_keys(url, num_keys, key_length):
    start_time = time.time()
    for i in range(num_keys):
        key = ''.join(random.choices('abcdefghijklmnopqrstuvwxyz', k=key_length))
        response = requests.put(f"{url}/{key}", data="value")
        if response.status_code != 200:
            print(f"Error putting key {key}: {response.status_code}")
    end_time = time.time()
    throughput = num_keys / (end_time - start_time)
    print(f"Put {num_keys} keys in {end_time - start_time:.2f} seconds, throughput: {throughput:.2f} keys/sec")


# response = requests.post("http://172.21.21.184:55557/join?nprime=" + nprime)
# response = requests.post("http://172.21.21.175:55555/leave")
# response = requests.get("http://172.21.21.176:55555/storage/1", data="hello world")
# put_random_keys("http://172.21.21.176:55555/storage", 1000, 10)
# get_random_keys("http://172.21.21.176:55555/storage", 1000, 10)
# response = requests.get("http://172.21.21.175:55555/node-info")
# print(response.status_code)
# print(response.text)

response = requests.get("http://172.21.21.188:55557/neighbors")
print(response.status_code)
print(response.text)

