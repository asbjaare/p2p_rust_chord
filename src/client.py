import requests

# response = requests.get('http://172.21.21.175:65000/storage/5')
# response = requests.put('http://172.21.21.175:65000/storage/6', data="Hallo_2")
response = requests.get('http://172.21.21.175:65000/storage/neighbors')
print(response.status_code)
print(response.text)