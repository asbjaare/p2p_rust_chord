
import requests

response = requests.get("http://172.21.21.188:55558/neighbors")
print(response.status_code)
print(response.text)
