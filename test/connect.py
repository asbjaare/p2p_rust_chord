
import requests
from time import sleep as wait
import time
import concurrent.futures

# key = input("what key:")
nprime = "172.21.21.175:60009"

start_time = time.time()

i = 0;
with open ("ip.txt", "r") as myfile:
    next(myfile)
    data=myfile.readlines()[:49]

    for line in data:
        ip_addr = line.rstrip()
        i = i+1
        print(ip_addr.split(" ")[2])
        print(i)
        # print("http://"+ip_addr.split(" ")[2]+":60021/join?nprime=" + nprime)
        response = requests.post("http://"+ip_addr.split(" ")[2]+":60009/join?nprime=" + nprime)
        nprime = ip_addr.split(" ")[2] + ":60009"
        print(response.status_code)
        print(response.text)
        # wait(0.1)



end_time = time.time()

tota_time = (end_time - start_time)

print(tota_time)

# response = requests.post("http://172.21.21.181:55552/join?nprime=" + nprime)
# print(response.status_code)
# print(response.text)