import requests
key = input('what key:')
response = requests.get('http://172.21.21.175:65000/storage/' + key)
print(response.status_code)
print(response.text)
