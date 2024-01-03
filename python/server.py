#!/usr/bin/env python

import click
import humanfriendly
import logging
import socket
import multiprocessing

BUCKET_SIZE = 2 ** 16


def echo(worker_id, client, address, data):
    sent = 0
    while sent < len(data):
        logging.debug('[{}] sending message ({}/{}) [@{}:{}]'.format(worker_id,
                      len(data) - sent, len(data), *address))
        sent = client.send(data[sent:])


def handle_client(worker_id, client, address):
    logging.info('[{}] handling client @ {}:{}'.format(worker_id, *address))
    encoded_size = client.recv(8)
    message_size = int.from_bytes(encoded_size, 'big')

    size_to_receive = message_size
    while size_to_receive > 0:
        logging.debug('[{}] receiving message ({}/{}) [@{}:{}]'.format(worker_id,
                      message_size - size_to_receive, message_size, *address))
        this_bucket = min(BUCKET_SIZE, size_to_receive)
        data = client.recv(this_bucket)
        if data:
            size_to_receive -= len(data)
            echo(worker_id, client, address, data)
        else:
            logging.warning(
                '[{}] failed to receive a message from client @ {}:{}'.format(worker_id, *address))
            break

    client.close()


def worker(worker_id, queue):
    while True:
        client, address = queue.get()
        try:
            handle_client(worker_id, client, address)
        except BaseException:
            logging.exception(
                '[{}] failed to handle client @ {}:{}'.format(worker_id, *address))


@click.command("server")
@click.argument("host", type=click.STRING, default="localhost")
@click.argument("port", type=click.INT, default=9090)
@click.option("--n-cores", "-j", type=click.INT,
              default=multiprocessing.cpu_count(), show_default=True)
@click.option("--backlog", '-b', type=click.INT, default=None)
@click.option("--verbose", '-v', is_flag=True, type=click.BOOL, default=False)
def server(host, port, n_cores, backlog, verbose):
    if verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    else:
        logging.getLogger().setLevel(logging.INFO)

    addr = (host, port)
    s = socket.create_server(addr, backlog=backlog)
    logging.info('Server started, listening @ {}:{}'.format(*addr))
    queue = multiprocessing.Queue()
    workers = [
        multiprocessing.Process(
            target=worker,
            args=(
                i + 1,
                queue)) for i in range(
            n_cores - 1)]
    for w in workers:
        w.start()

    while True:
        client, address = s.accept()
        queue.put((client, address))

    for w in workers:
        w.join()


if __name__ == "__main__":
    server()
