#!/usr/bin/env python

import click
import datetime
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

BUCKET_SIZE = 2 ** 15
BUCKET = b'\x42' * BUCKET_SIZE


def run_client(host, port, message_size) -> float:
    encoded_size = message_size.to_bytes(8, 'big')

    size_to_send = message_size

    addr = (host, port)
    server = socket.create_connection(addr)

    start_time = time.perf_counter()
    assert server.send(encoded_size) == 8

    while size_to_send > 0:
        this_bucket = min(BUCKET_SIZE, size_to_send)
        logging.debug(
            'sending message ({}/{})'.format(this_bucket, size_to_send))
        sent = server.send(BUCKET[:this_bucket])
        size_to_send -= sent

    received = 0
    while received < message_size:
        logging.debug('receiving message ({})'.format(message_size - received))
        bucket = server.recv(message_size - received)

        if bucket is None or len(bucket) == 0:
            logging.debug('empty echo')
            return -1

        logging.debug('message size: {}'.format(len(bucket)))
        received += len(bucket)
        if len(bucket.strip(b'\x42')) > 0:
            logging.debug("unsuccessful echo")
            return -1

    end_time = time.perf_counter()
    server.close()
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
@click.argument("port", default=9090, type=click.INT)
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
