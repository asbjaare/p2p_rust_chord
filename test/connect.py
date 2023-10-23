
import requests
from time import sleep as wait
import time

# key = input("what key:")
nprime = "172.21.21.175:55557"

i = 0;
with open ("ip.txt", "r") as myfile:
    next(myfile)
    data=myfile.readlines()[:19]

    for line in data:
        ip_addr = line.rstrip()
        i = i+1
        print(ip_addr.split(" ")[2])
        print(i)
        response = requests.post("http://"+ip_addr.split(" ")[2]+":55557/join?nprime=" + nprime)
        print(response.status_code)
        print(response.text)
        wait(0.1)

