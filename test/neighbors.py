
import requests

response = requests.get("http://172.21.21.117:60001/succ_list")
print(response.status_code)
print(response.text)
response = requests.get("http://172.21.21.117:60001/neighbors")
print(response.status_code)
print(response.text)
