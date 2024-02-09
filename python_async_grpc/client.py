#!/usr/bin/env python

import asyncio
import click
import datetime
import grpc
import humanfriendly
import logging
import multiprocessing
import pause
import pytimeparse2 as pytimeparse
import socket
import time
import uuid

import echo_pb2_grpc
import echo_pb2


async def run_client(host, port, stub, message_size, reporting: bool, end_timestamp: float):
    try:
        start_time = time.perf_counter()
        response = await stub.Echo(echo_pb2.EchoRequest(msg=b'\x42' * message_size))
        end_time = time.perf_counter()

        if reporting and end_time < end_timestamp:
            print('{:.3f}'.format(10**6 * (end_time - start_time)))
    except BaseException:
        logging.exception(
            'failed to run client workload @ {}:{}'.format(host, port))


async def run(id, host, port, n_cores, duration, warmup, message_size):
    async with grpc.aio.insecure_channel("{}:{}".format(host, port)) as channel:
        stub = echo_pb2_grpc.EchoerStub(channel)
        reporting = False
        start_time = time.perf_counter()
        end_timestamp = start_time + duration + warmup
        while time.perf_counter() < end_timestamp:
            await asyncio.gather(*[run_client(host, port, stub, message_size, reporting, end_timestamp) for _ in range(n_cores)])
            if not reporting and time.perf_counter() - start_time > warmup:
                reporting = True
                print('Start: {} {:.9f}'.format(id, time.perf_counter()))

    print('End: {} {:.9f}'.format(id, time.perf_counter()))


@click.command("client")
@click.argument("host", default="localhost", type=click.STRING)
@click.argument("port", default=9093, type=click.INT)
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
    pause.until(start)
    asyncio.run(run(id, host, port, n_cores, duration, warmup, message_size))


if __name__ == "__main__":
    client()
