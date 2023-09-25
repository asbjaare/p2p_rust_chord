import requests

# response = requests.get('http://172.21.21.175:65000/item/5')
response = requests.put('http://172.21.21.175:65000/item/5', data="Hallo")
print(response.status_code)
print(response.text)