
import requests
from time import sleep as wait
import time

nprime = "172.21.21.175:61021"

nodes = int(input("Enter the number of nodes: "))



i = 0;
with open ("ip.txt", "r") as myfile:
    next(myfile)
    data=myfile.readlines()[:nodes - 1]

    for line in data:
        ip_addr = line.rstrip()
        i = i+1
        response = requests.post("http://"+ip_addr.split(" ")[2]+":61021/join?nprime=" + nprime)
        nprime = ip_addr.split(" ")[2] + ":61021"
        print(response.text)



