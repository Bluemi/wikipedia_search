import argparse
import json
import os
import time

import deglib
import hnswlib
import numpy as np

from models import ModelPipeline
from utils import l2_normalize, quantize_data


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('index_type', choices=['hnsw', 'deglib'], help='the index type to use. Either "hnsw" or "deglib".')
    parser.add_argument('indir', type=str)
    parser.add_argument('--normalize', action='store_true', help='normalize vectors before adding to index')
    parser.add_argument('--quantize', action='store_true', help='quantize vectors before adding to index')
    return parser.parse_args()


class Index:
    def __init__(self, indir, index_type, dim: int, normalize: bool):
        self.index_type = index_type
        self.dim = dim
        if index_type == 'deglib':
            index = deglib.graph.load_readonly_graph(os.path.join(indir, 'index.deg'))
        else:
            metric = 'cosine' if normalize else 'l2'
            index = hnswlib.Index(space=metric, dim=dim)
            index.load_index(os.path.join(indir, 'index.hnsw'))
            index.set_ef(1200)
        self.index = index

    def search_query(self, query: np.ndarray):
        if self.index_type == 'deglib':
            indices, diffs = self.index.search(query, 0.2, 20)
        elif self.index_type == 'hnsw':
            indices, diffs = self.index.knn_query(query, k=20, filter=None)
        else:
            raise ValueError('Unknown index type: {}'.format(self.index_type))
        return indices, diffs


def main():
    args = parse_args()

    # read description
    with open(os.path.join(args.indir, 'description.json'), 'r') as f:
        description = json.load(f)

    dim = description['dim']

    # loading model
    print('loading model... ', end='', flush=True)
    # model = ModelPipeline.create_jina_embeddings_v3()
    model = ModelPipeline.create_jina_clip_v2()
    # model = ModelPipeline.create_mcip_vit_l14()
    print('done', flush=True)

    # loading index
    print('loading graph... ', end='', flush=True)
    index = Index(args.indir, args.index_type, dim, args.normalize)
    print('done', flush=True)

    # loading links
    print('loading links... ', end='', flush=True)
    with open(os.path.join(args.indir, 'links.txt'), 'r') as f:
        links = f.read().split('\n')
    print('done', flush=True)

    while True:
        search_text = input('\nEnter search text: ')
        start_time = time.perf_counter()
        if not search_text:
            break
        search_feature = model(search_text)
        if args.normalize:
            search_feature = l2_normalize(search_feature)
        if args.quantize:
            search_feature = quantize_data(search_feature, max_val=0.4)
        indices, diffs = index.search_query(search_feature)
        end_time = time.perf_counter()
        for i, d in zip(indices[0], diffs[0]):
            print(links[i], d)
        print('results in {:.3f}s'.format(end_time - start_time), flush=True)


if __name__ == '__main__':
    main()
