# This file can be used to test the brute force protection on the auth endpoint.
# If successful, the script should get a 400 5 times (or less, if the bucket is
# not full because of previous requests). After that, you should only see 429s.
import requests

login_data = {
    "username": "admin",
    "password": "password"
}

def brute_login(url):
    response = requests.post(url, data=login_data)
    print(response.status_code)

while True:
    brute_login("http://127.0.0.1:8000/api/auth/login")
