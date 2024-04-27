# A first draft, proof-of-concept for how Imagehawk will work.
# Doing this in Python because I have no experience with APIs and want to get
# my feet wet in a language I'm more familiar with than Rust.

# To use this script, make sure you have the "requests" library installed.
# It can be installed with "pip install requests"

from requests import get
import os
import json
# print("Hello world!")
def grabFromDockerHub(image):
    namespace, repo = image.split("/")
    response = get(f"https://hub.docker.com/v2/namespaces/{namespace}/repositories/{repo}/tags/latest")
    if response.status_code == 200:
        response_json = response.json()
        digest = response_json.get("digest")
        print(f"Digest: {digest}")
    
    local = os.system(f"docker image inspect {image}")
    local.get("RepoDigests")



grabFromDockerHub("pihole/pihole")
