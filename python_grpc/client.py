#!/usr/bin/env python

import click
import datetime
import grpc
import humanfriendly
import logging
import multiprocessing
import pause
import pytimeparse2 as pytimeparse
import signal
import socket
import sys
import time
import uuid


import echo_pb2_grpc
import echo_pb2


def run_client(host, port, message_size) -> float:
    logging.debug("trying to connect to {}:{}".format(host, port))
    with grpc.insecure_channel("{}:{}".format(host, port)) as channel:
        logging.debug("connected to {}:{}".format(host, port))
        stub = echo_pb2_grpc.EchoerStub(channel)
        start_time = time.perf_counter()
        response = stub.Echo(echo_pb2.EchoRequest(msg=b'\x42' * message_size))
        end_time = time.perf_counter()

    logging.debug("connected to {}:{}".format(host, port))
    return end_time - start_time


def run_worker(id, host, port, message_size, duration, warmup):
    worker_start = time.perf_counter()
    reporting = False
    while time.perf_counter() - worker_start < duration + warmup:
        try:
            elapsed = run_client(host, port, message_size)
            ts = time.perf_counter() - worker_start
            if not reporting and ts > warmup:
                reporting = True
                print('Start: {} {:.9f}'.format(id, time.perf_counter()))

            if reporting and ts < duration + warmup:
                print('{:.3f}'.format(10**6 * elapsed))

        except BaseException:
            logging.exception(
                'failed to run client workload @ {}:{}'.format(host, port))

    print('End: {} {:.9f}'.format(id, time.perf_counter()))


@click.command("client")
@click.argument("host", default="localhost", type=click.STRING)
@click.argument("port", default=9092, type=click.INT)
@click.option("--n-cores", "-j", type=click.INT,
              default=multiprocessing.cpu_count(), show_default=True)
@click.option("--duration", '-d', type=click.STRING, default='60s',
              help='duration of the experiment in a human-readable format')
@click.option("--warmup", '-w', type=click.STRING, default='10s',
              help='duration of the warmup phase in a human-readable format')
@click.option("--start",
              '-s',
              type=click.DateTime(formats=["%H:%M:%S"]),
              default=datetime.datetime.fromtimestamp(time.time()).strftime('%H:%M:%S'),
              help='start timestamp of the experiment (useful for synchronizing multiple clients')
@click.option("--message-size",
              "-m",
              default="1",
              show_default=True,
              type=click.STRING,
              help="message size in B")
@click.option("--verbose", '-v', is_flag=True, type=click.BOOL, default=False)
def client(
        host,
        port,
        n_cores,
        duration,
        warmup,
        start,
        message_size,
        verbose):
    message_size = humanfriendly.parse_size(message_size, binary=True)
    duration = pytimeparse.parse(duration)
    warmup = pytimeparse.parse(warmup)
    today = datetime.date.today()
    start = start.replace(year=today.year, month=today.month, day=today.day)
    id = '{}:{}'.format(socket.gethostname(), uuid.uuid4().hex)

    if verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    else:
        logging.getLogger().setLevel(logging.INFO)

    print('Message Size: {}'.format(message_size))

    pool = multiprocessing.Pool(
        n_cores, lambda: signal.signal(
            signal.SIGINT, signal.SIG_IGN))
    pause.until(start)
    start_time = time.perf_counter()
    try:
        for _ in range(n_cores):
            pool.apply_async(
                run_worker,
                args=(
                    id,
                    host,
                    port,
                    message_size,
                    duration,
                    warmup))

        pool.close()
        pool.join()

    except KeyboardInterrupt:
        print('Terminating', file=sys.stderr)
        pool.terminate()
        pool.join()


if __name__ == "__main__":
    client()
