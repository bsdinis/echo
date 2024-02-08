#!/usr/bin/env python

import asyncio
import click
import datetime
import grpc
import humanfriendly
import logging
import multiprocessing
import pause
import time

import echo_pb2_grpc
import echo_pb2


async def run_client(host, port, stub, message_size):
    try:
        start_time = time.perf_counter()
        response = await stub.Echo(echo_pb2.EchoRequest(msg=b'\x42' * message_size))
        end_time = time.perf_counter()

        print('{:.3f}'.format(10**6 * (end_time - start_time)))
    except BaseException:
        logging.exception(
            'failed to run client workload @ {}:{}'.format(host, port))


async def run(host, port, n_cores, reps, message_size, start):
    async with grpc.aio.insecure_channel("{}:{}".format(host, port)) as channel:
        stub = echo_pb2_grpc.EchoerStub(channel)
        start_time = time.perf_counter()
        await asyncio.gather(*[run_client(host, port, stub, message_size) for _ in range(reps)])
        end_time = time.perf_counter()

    print('Elapsed: {:.9f}'.format((end_time - start_time)))
    print('Message Size: {}'.format(message_size))


@click.command("client")
@click.argument("host", default="localhost", type=click.STRING)
@click.argument("port", default=9093, type=click.INT)
@click.option("--n-cores", "-j", type=click.INT,
              default=None, show_default=True)
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

    if verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    else:
        logging.getLogger().setLevel(logging.INFO)

    pause.until(start)
    asyncio.run(run(host, port, n_cores, reps, message_size))


if __name__ == "__main__":
    client()
