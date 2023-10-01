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
key = "2"
# print("Generating key %s" % key)
response = requests.get('http://172.21.21.177:55000/storage/' + key)
# response = requests.put('http://172.21.21.177:55000/storage/' + key, data="Hallo_2")
# response = requests.get('http://172.21.21.177:55000/storage/neighborkey')
print(response.status_code)
print(response.text)