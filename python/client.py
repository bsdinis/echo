#!/usr/bin/env python

import click
import datetime
import humanfriendly
import logging
import multiprocessing
import pause
import socket
import time

BUCKET_SIZE = 2 ** 15
BUCKET = b'\x42' * BUCKET_SIZE


def run_client(host, port, message_size):
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
    print('{:.3f}'.format(10**6 * (end_time - start_time)))


def run_worker(host, port, message_size, reps):
    for _ in range(reps):
        try:
            run_client(host, port, message_size)
        except BaseException:
            logging.exception(
                'failed to run client workload @ {}:{}'.format(host, port))


@click.command("client")
@click.argument("host", default="localhost", type=click.STRING)
@click.argument("port", default=9090, type=click.INT)
@click.option("--n-cores", "-j", type=click.INT,
              default=multiprocessing.cpu_count(), show_default=True)
@click.option("--reps", "-r", type=click.INT, default=1000, show_default=True)
@click.option("--message-size",
              "-s",
              default="1",
              show_default=True,
              type=click.STRING,
              help="message size in B")
@click.option("--start",
              type=click.DateTime(formats=["%H:%M:%S"]),
              default=datetime.datetime.fromtimestamp(time.time()).strftime('%H:%M:%S'))
@click.option("--verbose", '-v', is_flag=True, type=click.BOOL, default=False)
def client(host, port, n_cores, reps, message_size, start, verbose):
    message_size = humanfriendly.parse_size(message_size, binary=True)
    today = datetime.date.today()
    start = start.replace(year=today.year, month=today.month, day=today.day)

    workers = [
        multiprocessing.Process(
            target=run_worker, args=(
                host, port, message_size, reps // n_cores)) for _ in range(
            n_cores - 1)] + [
        multiprocessing.Process(
            target=run_worker, args=(
                host, port, message_size, reps // n_cores + reps %
                n_cores))]

    pause.until(start)
    start_time = time.perf_counter()
    for w in workers:
        w.start()
    for w in workers:
        w.join()
    end_time = time.perf_counter()

    print('Elapsed: {:.9f}'.format((end_time - start_time)))
    print('Message Size: {}'.format(message_size))


if __name__ == "__main__":
    client()
