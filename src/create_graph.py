import argparse
import json
import os
import time
from typing import Iterator

import hnswlib
import numpy as np
import deglib
from tqdm import tqdm

from utils import l2_normalize

CHUNK_SIZE = 1024


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('index_type', choices=['hnsw', 'deglib'], help='the index type to use. Either "hnsw" or "deglib".')
    parser.add_argument('indir')
    parser.add_argument('--normalize', action='store_true', help='normalize vectors before adding to index')
    return parser.parse_args()


def main():
    args = parse_args()

    # read description
    with open(os.path.join(args.indir, 'description.json'), 'r') as f:
        description = json.load(f)

    dim, num_samples = description['dim'], description['num_samples']

    print('num_samples={}  dim={}'.format(num_samples, dim))

    if args.index_type == 'hnsw':
        index = build_hnsw_index(dim, num_samples, os.path.join(args.indir, 'features.bin'), args.normalize)
        index.save_index(os.path.join(args.indir, 'index.hnsw'))
    elif args.index_type == 'deglib':
        index = build_deglib_from_data(dim, num_samples, os.path.join(args.indir, 'features.bin'), args.normalize)
        index.save_graph(os.path.join(args.indir, 'index.deg'))


def get_num_pages(indir):
    with open(os.path.join(indir, 'links.txt'), 'r') as f:
        links = f.read().split('\n')
    pages = set(l.split('#')[0] for l in links)
    return len(pages)


def iterate_chunks(data_file: str, num_samples: int, dim: int, normalize: bool) -> Iterator[np.ndarray]:
    with open(data_file, 'rb') as df:
        for min_index in range(0, num_samples, CHUNK_SIZE):
            data_bytes = df.read(dim * CHUNK_SIZE * 4)
            chunk = np.frombuffer(data_bytes, dtype=np.float32).reshape(-1, dim)
            if normalize:
                chunk = l2_normalize(chunk)
            yield chunk


def build_hnsw_index(dim, num_samples, data_file: str, normalize: bool):
    metric = 'cosine' if normalize else 'l2'
    index = hnswlib.Index(space=metric, dim=dim)
    index.init_index(max_elements=num_samples, ef_construction=400, M=24)
    index.set_num_threads(6)

    n_chunks = num_samples // CHUNK_SIZE + 1

    start_time = time.perf_counter()
    for chunk in tqdm(iterate_chunks(data_file, num_samples, dim, normalize), total=n_chunks):
        index.add_items(chunk)
    print('Added {} data points after {:5.1f}s\n'.format(num_samples, time.perf_counter() - start_time), flush=True)

    return index


def build_deglib_from_data(dim: int, num_samples: int, data_file: str, normalize: bool) -> deglib.graph.SizeBoundedGraph:
    graph = deglib.graph.SizeBoundedGraph.create_empty(num_samples, dim, 24, deglib.Metric.L2)
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
            data_bytes = df.read(dim * CHUNK_SIZE * 4)
            chunk = np.frombuffer(data_bytes, dtype=np.float32).reshape(-1, dim)

            if normalize:
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
