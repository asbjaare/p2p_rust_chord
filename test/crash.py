import requests

response = requests.post("http://172.21.21.117:60021/sim-crash")
response = requests.post("http://172.21.21.178:60021/sim-crash")
response = requests.post("http://172.21.21.180:60021/sim-crash")
response = requests.post("http://172.21.21.119:60021/sim-crash")
response = requests.post("http://172.21.21.108:60021/sim-crash")
response = requests.post("http://172.21.21.200:60021/sim-crash")
print(response.status_code)
print(response.text)
