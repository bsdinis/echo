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
- `-d`, `--duration`: duration of the experiment
- `-w`, `--warmup`: duration of the warmup cycle
- `-s`, `--start`: start instatnt
- `-m`, `--message-size`: size of the message to send (parseable, like `1MB` or `256KiB`)

## Output

The clients SHALL output a list of latencies in microseconds.
There MUST be at least one line with `Messagte Size: Z`, in bytes. In the event there are multiple such lines, they should be identical.
Each client will output a `Start: <ID> A.B` and an `End: <ID> X.Y`, such that `X.Y - A.B` will give the elapsed time in seconds. In the event of multiple `Start`s and `End`s per `<ID>`, the considered `Start` will be the minimum value and the considered `End` the maximum value.
