#!/usr/bin/env python

import click
import grpc
import logging
import multiprocessing
from concurrent import futures

import echo_pb2_grpc
import echo_pb2


class Echoer(echo_pb2_grpc.EchoerServicer):
    def Echo(self, request, context):
        return echo_pb2.EchoReply(msg=request.msg)


def serve(host, port, n_cores):
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=n_cores))
    echo_pb2_grpc.add_EchoerServicer_to_server(Echoer(), server)
    server.add_insecure_port("{}:{}".format(host, port))
    server.start()
    logging.info("Server started, listening @ {}:{}".format(host, port))
    server.wait_for_termination()


@click.command("server")
@click.argument("host", type=click.STRING, default="localhost")
@click.argument("port", type=click.INT, default=9092)
@click.option("--n-cores", "-j", type=click.INT,
              default=multiprocessing.cpu_count(), show_default=True)
@click.option("--verbose", '-v', is_flag=True, type=click.BOOL, default=False)
def server(host, port, n_cores, verbose):
    if verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    else:
        logging.getLogger().setLevel(logging.INFO)

    return serve(host, port, n_cores)


if __name__ == "__main__":
    server()
