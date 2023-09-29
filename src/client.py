import requests
import argparse
import http.client
import json
import random
import textwrap
import uuid


# conn = http.client.HTTPConnection("172.21.21.178:55000")
# conn.request("GET", "/storage/neighbors")
# res = conn.getresponse()
# print(res.status, res.reason)
# data = res.read()
# print(data.decode("utf-8"))
key = str(uuid.uuid4())
response = requests.get('http://172.21.21.175:55000/storage/5')
# response = requests.put('http://172.21.21.175:55000/storage/5', data="Hallo_2")
# response = requests.get('http://172.21.21.177:55000/storage/neighbors')
print(response.status_code)
print(response.text)