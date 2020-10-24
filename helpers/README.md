# Helpers

Just a few python scripts I've used during development

`requirements.txt` has the dependencies

## dup_check.py

Connects to blaseball and looks for duplicate messages or games

Used to sanity check `event_source.rs`

## event_server.py

Can record an arbitrary event source to a given file, and then
replay that file back as an event source on `localhost:5000`

For when blaseball takes a siesta