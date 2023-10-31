
import requests

response = requests.get("http://172.21.21.187:60021/succ_list")
print(response.status_code)
print(response.text)
