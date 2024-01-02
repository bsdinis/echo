# Echo Server comparison

Comparing different PLs/frameworks as it pertains to the performance of simple echo servers.

## API

### Server

The server should support the following CLI options:
- `[hostname]`: hostname to listen on
- `[port]`: port to listen on
- `-j`, `--n-cores`: integer, number of cores to use (default: number of cores in the machine)
- `-b`, `--backlog`: integer, backlog of the listening socket

### Client

The client should support the following CLI options:
- `[hostname]`: hostname to connect to
- `[port]`: port to connect to
- `-j`, `--n-cores`: integer, number of cores to use for concurrency (default: number of cores in the machine)
- `-r`, `--reps`: integer, number of requests to send in total (default: 1000)
- `-s`, `--message-size`: size of the message to send (parseable, like `1MB` or `256KiB`)


