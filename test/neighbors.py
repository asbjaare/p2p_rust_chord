
import requests

response = requests.post("http://172.21.21.188:55557/neighbors")
print(response.status_code)
print(response.text)
