# Python Echo Server

Simple [`socket`](https://docs.python.org/3/library/socket.html)-based server-client for profiling.

# Server

```
Usage: server.py [OPTIONS] [HOST] [PORT]

Options:
  -j, --n-cores INTEGER  [default: 16]
  -b, --backlog INTEGER
  -v, --verbose
  --help                 Show this message and exit.
```

# Client

```
Usage: client.py [OPTIONS] [HOST] [PORT]

Options:
  -j, --n-cores INTEGER    [default: 16]
  -r, --reps INTEGER       [default: 1000]
  -s, --message-size TEXT  message size in B  [default: 1]
  -v, --verbose
  --help                   Show this message and exit.
```
