import argparse
import os

import deglib

from models import ModelPipeline
from utils import l2_normalize

NORMALIZE = True


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('indir', type=str)
    return parser.parse_args()


def main():
    args = parse_args()
    print('loading model... ', end='', flush=True)
    # model = ModelPipeline.create_e5_base_sts_en_de()
    model = ModelPipeline.create_jina_embeddings_v3()
    print('done', flush=True)
    print('loading graph... ', end='', flush=True)
    graph = deglib.graph.load_readonly_graph(os.path.join(args.indir, 'index.deg'))
    print('done', flush=True)
    print('loading links... ', end='', flush=True)
    with open(os.path.join(args.indir, 'links.txt'), 'r') as f:
        links = f.read().split('\n')
    print('done', flush=True)

    while True:
        search_text = input('Enter search text: ')
        if not search_text:
            break
        search_feature = model(search_text)
        if NORMALIZE:
            search_feature = l2_normalize(search_feature)
        indices, diffs = graph.search(search_feature, 0.2, 20)
        indices = indices[0]
        for i in indices:
            print(links[i])


if __name__ == '__main__':
    main()
