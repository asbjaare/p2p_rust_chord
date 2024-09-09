import requests
import argparse
import http.client
import json
import random
import textwrap
import uuid


key = "2"
response = requests.get("http://172.21.21.177:55000/storage/" + key)
print(response.status_code)
print(response.text)
