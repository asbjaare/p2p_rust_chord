import requests

# key = input("what key:")
nprime = "172.21.21.176:55000"
response = requests.get("http://172.21.21.175:55000/node-info")
print(response.status_code)
print(response.text)
