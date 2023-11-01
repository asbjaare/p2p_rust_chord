import requests
from time import sleep as wait
import time

    
start = time.time()
nodes = 5
i = 1;
with open ("ip.txt", "r") as myfile:

    data=myfile.readlines()[:nodes]
    for line in data:
        ip_addr = line.rstrip()
     
        # print(i)
        response = requests.post("http://"+ip_addr.split(" ")[2]+":61021/leave")
        i = i + 1
        # print(response.status_code)
        # print(response.text)
        # wait(0.5)

end = time.time()

print(f"{end - start},{nodes*2}")


