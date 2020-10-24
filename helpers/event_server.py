#!/bin/env python3

import flask
import time
import argparse
import sseclient

app = flask.Flask(__name__)
app.debug = True
LINES = []
DELAY = 2


def chunk(s):
    return f"{hex(len(s))[2:]}\r\n{s}\r\n"


def stream_lines():
    is_first = True
    for line in LINES:
        if not is_first:
            time.sleep(DELAY)
        else:
            is_first = False

        yield chunk(f"data: {line}\n\n")
    yield chunk("")


@app.route("/stream")
def serve_stream():
    r = flask.Response(stream_lines(), mimetype="text/event-stream")
    r.headers["Transfer-Encoding"] = "chunked"
    return r


def record(url, fname, overwrite):
    if overwrite:
        print(f"Overwriting {fname}")
    else:
        print(f"Recording to {fname}")

    try:
        with open(fname, "w" if overwrite else "a") as output:
            for message in sseclient.SSEClient(url):
                print(f"Received line of len {len(message.data)}")
                output.write(message.data + "\n")
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Record and replay an event source")
    subparsers = parser.add_subparsers()
    record_parser = subparsers.add_parser("record")
    record_parser.add_argument("url")
    record_parser.add_argument("file")
    record_parser.add_argument("-o", "--overwrite", action="store_true")
    record_parser.set_defaults(is_record=True)

    replay_parser = subparsers.add_parser("replay")
    replay_parser.add_argument("file")
    replay_parser.set_defaults(is_record=False)
    replay_parser.add_argument(
        "-d",
        "--delay",
        default=DELAY,
        type=int,
        help="Delay between serving lines, in seconds. Default 2 seconds"
    )

    args = parser.parse_args()

    if args.is_record:
        record(args.url, args.file, args.overwrite)
    else:
        DELAY = args.delay
        with open(args.file, "r") as f:
            LINES = [line.rstrip() for line in f.readlines()]
        app.run(threaded=True)
