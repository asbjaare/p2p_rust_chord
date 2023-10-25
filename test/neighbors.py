
import requests

response = requests.get("http://172.21.21.119:55550/reps_keys")
print(response.status_code)
print(response.text)
