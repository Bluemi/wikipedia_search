import argparse
import os
import time

import numpy as np
import deglib

from models import l2_normalize

# DIM = 768
DIM = 1024
CHUNK_SIZE = 1024
NORMALIZE = True


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('indir')
    return parser.parse_args()


def main():
    args = parse_args()
    num_samples = get_num_samples(args.indir)
    num_pages = get_num_pages(args.indir)
    print('num_pages={} num_samples={}'.format(num_pages, num_samples))

    graph = build_deglib_from_data(os.path.join(args.indir, 'features.bin'), num_samples)
    graph.save_graph(os.path.join(args.indir, 'index.deg'))


def get_num_samples(indir):
    with open(os.path.join(indir, 'links.txt'), 'r') as f:
        links = f.read()
        return len(links.split('\n'))


def get_num_pages(indir):
    with open(os.path.join(indir, 'links.txt'), 'r') as f:
        links = f.read().split('\n')
    pages = set(l.split('#')[0] for l in links)
    return len(pages)


def build_deglib_from_data(data_file: str, num_samples: int, _quantize: bool = False) -> deglib.graph.SizeBoundedGraph:
    graph = deglib.graph.SizeBoundedGraph.create_empty(num_samples, DIM, 24, deglib.Metric.L2)
    builder = deglib.builder.EvenRegularGraphBuilder(
        graph, rng=None, lid=deglib.builder.LID.High, extend_k=32, extend_eps=0.1, improve_k=0
    )

    print(f"Start adding {num_samples} data points to builder", flush=True)
    start_time = time.perf_counter()
    labels = np.arange(num_samples, dtype=np.uint32)

    with open(data_file, 'rb') as df:
        for counter, min_index in enumerate(range(0, num_samples, CHUNK_SIZE)):
            if counter != 0 and counter % 10 == 0:
                print('Added {} data points after {:5.1f}s'.format(min_index, time.perf_counter() - start_time), flush=True)

            max_index = min(min_index + CHUNK_SIZE, num_samples)
            data_bytes = df.read(DIM * CHUNK_SIZE * 4)
            chunk = np.frombuffer(data_bytes, dtype=np.float32).reshape(-1, DIM)

            if NORMALIZE:
                chunk = l2_normalize(chunk)

            builder.add_entry(
                labels[min_index:max_index],
                chunk
            )
            print('added chunk with shape: {}'.format(chunk.shape))
    print('Added {} data points after {:5.1f}s\n'.format(num_samples, time.perf_counter() - start_time), flush=True)

    print('Start building graph:', flush=True)
    builder.build(callback='progress')

    # remove builder to free memory
    del builder

    print('Removing all none MRNG conform edges ... ', flush=True)
    graph.remove_non_mrng_edges()

    return graph


if __name__ == '__main__':
    main()
