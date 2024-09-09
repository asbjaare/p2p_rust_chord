
import requests

response = requests.get("http://172.21.21.175:60009/node-info")
print(response.status_code)
print(response.text)
response = requests.get("http://172.21.21.175:60009/neighbors")
print(response.status_code)
print(response.text)
