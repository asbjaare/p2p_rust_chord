import requests

# key = input("what key:")
nprime = "172.21.21.176:55555"
# response = requests.post("http://172.21.21.182:55555/join?nprime=" + nprime)

response = requests.get("http://172.21.21.178:55555/neighbors")
# response = requests.get("http://172.21.21.175:55555/node-info")
print(response.status_code)
print(response.text)
