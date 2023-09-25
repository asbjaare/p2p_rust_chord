import requests

response = requests.get('http://172.21.21.175:65000/')
print(response.status_code)
print(response.text)