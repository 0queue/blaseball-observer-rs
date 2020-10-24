#!/bin/env python3
import json

import sseclient

messages = sseclient.SSEClient("https://www.blaseball.com/events/streamData")

last_data = ""
last_game = {}
for m in messages:
    if last_data == m.data:
        print("DUPLICATE DATA")
    else:
        print("changed data")

    j = json.loads(m.data)

    schedule = j["value"]["games"]["schedule"]

    g = next(s for s in schedule if s["awayTeamNickname"] == "Lift" or s["homeTeamNickname"] == "Lift")

    if last_game == g:
        print("DUPLICATE GAME\n\n")
    else:
        print("changed game\n\n")

    last_game = g
    last_data = m.data
