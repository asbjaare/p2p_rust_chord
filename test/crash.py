import requests

response = requests.post("http://172.21.21.177:55550/sim-crash")
print(response.status_code)
print(response.text)
