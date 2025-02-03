import argparse
import json
import os

import deglib
import hnswlib

from models import ModelPipeline
from utils import l2_normalize


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('index_type', choices=['hnsw', 'deglib'], help='the index type to use. Either "hnsw" or "deglib".')
    parser.add_argument('indir', type=str)
    parser.add_argument('--normalize', action='store_true', help='normalize vectors before adding to index')
    return parser.parse_args()


def main():
    args = parse_args()

    # read description
    with open(os.path.join(args.indir, 'description.json'), 'r') as f:
        description = json.load(f)

    dim = description['dim']

    # loading model
    print('loading model... ', end='', flush=True)
    model = ModelPipeline.create_jina_embeddings_v3()
    print('done', flush=True)

    # loading index
    print('loading graph... ', end='', flush=True)
    if args.index_type == 'deglib':
        index = deglib.graph.load_readonly_graph(os.path.join(args.indir, 'index.deg'))
    else:
        metric = 'cosine' if args.normalize else 'l2'
        index = hnswlib.Index(space=metric, dim=dim)
        index.load_index(os.path.join(args.indir, 'index.hnsw'))
    print('done', flush=True)

    # loading links
    print('loading links... ', end='', flush=True)
    with open(os.path.join(args.indir, 'links.txt'), 'r') as f:
        links = f.read().split('\n')
    print('done', flush=True)

    while True:
        search_text = input('Enter search text: ')
        if not search_text:
            break
        search_feature = model(search_text)
        if args.normalize:
            search_feature = l2_normalize(search_feature)
        if args.index_type == 'deglib':
            indices, diffs = index.search(search_feature, 0.2, 20)
        elif args.index_type == 'hnsw':
            indices, diffs = index.knn_query(search_feature, k=20, filter=None)
        else:
            raise ValueError('Unknown index type: {}'.format(args.index_type))
        indices = indices[0]
        for i in indices:
            print(links[i])


if __name__ == '__main__':
    main()
