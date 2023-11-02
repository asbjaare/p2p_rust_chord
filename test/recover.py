import requests

response = requests.post("http://172.21.21.178:61021/sim-recover")
print(response.status_code)
print(response.text)
