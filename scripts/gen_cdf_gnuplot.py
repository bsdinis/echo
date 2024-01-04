#!/usr/bin/env python

import click
import csv
import hashlib
import logging
import sys

from typing import *


def generate_map(filename: str) -> Dict[str, str]:
    mapping = dict()
    if filename:
        with open(filename) as csvfile:
            reader = csv.reader(csvfile, delimiter=' ')
            for row in reader:
                if len(row) == 0:
                    continue
                elif len(row) == 1:
                    raise ValueError('row with a single element: {}', row[0])
                elif len(row) > 2:
                    raise ValueError(
                        'row with more than 2 elements ({}): {}', len(row), row)

                mapping[row[0]] = row[1]

    return mapping


def gen_output_filename(dat_files):
    hasher = hashlib.sha3_256()
    for d in dat_files:
        hasher.update(d.encode('utf-8'))
    return '{}.eps'.format(hasher.hexdigest()[:20])


def print_gnuplot_header(output_filename):
    print('''reset

set terminal postscript eps colour size 12cm,8cm enhanced font 'Helvetica,20'
set output '{}'

set border linewidth 0.75
set key outside above


# set axis
set tics scale 0.75
set xlabel 'Transfer Latency (μ)'
set ylabel 'CDF'
set xrange [0:*]
set yrange [0:100]

'''.format(output_filename))


def title(dat_file, translator):
    dat_file = dat_file.split('/')[-1]
    if dat_file in translator:
        logging.info(
            'translating {} -> {} [map]'.format(dat_file, translator[dat_file]))
        return translator[dat_file]

    title = dat_file.replace('_', ' ').replace('.dat', '')
    logging.info('translating {} -> {} [default]'.format(dat_file, title))
    return title


def print_cdf_plot(dat_files, translator):
    print(
        'plot {}'.format(
            ','.join(
                "'{}' using 2:1 with lines title '{}'".format(
                    dat_file,
                    title(
                        dat_file,
                        translator)) for dat_file in dat_files)))


@click.command('gen_cdf_gnuplot',
               help='generate a gnuplot file for a cdf, from a number of log/dat files and a translation mapping in a csv')
@click.argument('dat-files', nargs=-1, type=click.Path(exists=True))
@click.option('--map-file', '-m', type=click.Path(exists=True),
              help='csv file with mapping for renaming labels in the cdf. the format is <filename>, <label>')
@click.option('--eps-filename', '-e', type=click.STRING,
              help='use this as the eps prefix')
def gen_cdf_gnuplot(dat_files, map_file, eps_filename):
    if len(dat_files) == 0:
        return

    translator = generate_map(map_file)

    output_filename = eps_filename or gen_output_filename(dat_files)
    print_gnuplot_header(output_filename)
    print_cdf_plot(dat_files, translator)


if __name__ == '__main__':
    logging.basicConfig(stream=sys.stderr, level=logging.INFO)
    gen_cdf_gnuplot()
