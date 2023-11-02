import requests

# response = requests.post("http://172.21.21.176:60035/sim-recover")
response = requests.post("http://172.21.21.178:60009/sim-crash")
response = requests.post("http://172.21.21.180:60009/sim-crash")
response = requests.post("http://172.21.21.119:60009/sim-crash")
response = requests.post("http://172.21.21.108:60009/sim-crash")
# response = requests.post("http://172.21.21.200:60009/sim-crash")
# response = requests.post("http://172.21.21.120:60009/sim-crash")
# response = requests.post("http://172.21.21.111:60009/sim-crash")
# response = requests.post("http://172.21.21.126:60029/sim-crash")
# response = requests.post("http://172.21.21.176:60029/sim-crash")
print(response.status_code)
print(response.text)
