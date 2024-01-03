#!/usr/bin/env python

import click
import humanfriendly
import logging
import socket
import time
import multiprocessing

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
    return end_time - start_time


def run_worker(host, port, message_size, queue, reps):
    for _ in range(reps):
        try:
            elapsed = run_client(host, port, message_size)
            queue.put(elapsed)
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
@click.option("--verbose", '-v', is_flag=True, type=click.BOOL, default=False)
def client(host, port, n_cores, reps, message_size, verbose):
    message_size = humanfriendly.parse_size(message_size, binary=True)

    if verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    else:
        logging.getLogger().setLevel(logging.INFO)

    queue = multiprocessing.Queue(reps)
    workers = [
        multiprocessing.Process(
            target=run_worker, args=(
                host, port, message_size, queue, reps // n_cores)) for _ in range(
            n_cores - 1)] + [
        multiprocessing.Process(
            target=run_worker, args=(
                host, port, message_size, queue, reps // n_cores + reps %
                n_cores))]

    start_time = time.perf_counter()
    for w in workers:
        w.start()
    for w in workers:
        w.join()
    end_time = time.perf_counter()

    elapsed = list()
    while queue.empty() == False:
        elapsed.append(queue.get())
    print('\n'.join('{:.3f}'.format(10**6 * e) for e in elapsed))
    print('Throughput: {:.3f}'.format(
        (len(elapsed) * message_size) / (end_time - start_time)))


if __name__ == "__main__":
    client()
