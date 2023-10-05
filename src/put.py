import requests
key = input('what key:')
response = requests.put('http://172.21.21.175:65000/storage/' + key, data=key)
print(response.status_code)
print(response.text)
