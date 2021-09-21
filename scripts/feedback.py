#!/bin/env /bin/python3.9

import requests
import getopt
import sys
import json
import creds

SERVER = "http://localhost:7000"
USER = ""
PASS = ""

CAMPAIGN_NAME ="feedback script flood new"
CAMPAIGN =  ""
COOKIE = creds.COOKIE

client = requests.Session()

def login():
    url = SERVER + "/api/v1/signin"
    creds = {
        "login": USER,
        "password": PASS,
    }
    resp = client.post(url, json=creds)
    if resp.status_code == 200:
        print("[*] Authentication successful")

def create_campaign():
    url = SERVER + "/api/v1/campaign/new"
    payload = {"name": CAMPAIGN_NAME}

    resp = client.post(url, json=payload, cookies=COOKIE)
    if resp.status_code == 200:
        data = resp.json()
        global CAMPAIGN
        CAMPAIGN = data["uuid"]
        print("[*] Campaign creation successful. ID: %s" % (CAMPAIGN))

def rating(num: int):
    url = format("%s/api/v1/feedback/%s/rating" % (SERVER, CAMPAIGN))
    payload = {
            "helpful": True,
            "description": "Lorem ipsum dolor sit amet",
            "page_url": url
            }
    resp = client.post(url, json=payload, cookies=COOKIE)
    def err(r):
        if r.status_code != 200:
            data = resp.json()
            print("[*] Feedback submission failure ID: %s" % (resp.text))

    err(resp)
    payload["description"] = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
    for _ in range(0,num):
        resp = client.post(url, json=payload, cookies=COOKIE)
        err(resp)

if __name__ == "__main__":
    argv = sys.argv[1:]
    try:
        opts, args = getopt.getopt(argv, "s:u:p:")
    except:
        print("error")

    for opt, arg in opts:
        if opt in ["-s"]:
            SERVER = arg
        elif opt in ["-u"]:
            USER = arg
        elif opt in ["-p"]:
            PASS = arg
        else:
            pass

    print("user: %s pass: %s URI: %s" % (USER, PASS, SERVER))
#    login()
    create_campaign()
    rating(30)
