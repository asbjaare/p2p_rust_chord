import requests

response = requests.post("http://172.21.21.188:55558/sim-crash")
print(response.status_code)
print(response.text)
