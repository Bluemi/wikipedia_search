import argparse
import dataclasses
import json
import os
import time

import deglib
import hnswlib
import numpy as np

from models import load_model
from tables import Table
from utils import l2_normalize, quantize_data


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('indir', type=str)
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

    def search_query(self, query: np.ndarray, k: int = 20):
        if self.index_type == 'deglib':
            indices, diffs = self.index.search(query, 0.2, k)
        elif self.index_type == 'hnsw':
            indices, diffs = self.index.knn_query(query, k=k, filter=None)
        else:
            raise ValueError('Unknown index type: {}'.format(self.index_type))
        return indices, diffs


@dataclasses.dataclass
class ResultEntry:
    title: str
    link: str
    distance: float
    views: int

    def sort_key(self):
        # return self.distance / 200 - np.sqrt(self.views)
        view_factor = ((1000 / (1000 + self.views)) - 1) * 0.6 + 1
        return self.distance * view_factor


def main():
    args = parse_args()

    # read description
    with open(os.path.join(args.indir, 'description.json'), 'r') as f:
        description = json.load(f)

    dim = description['dim']
    model_name = description['model']
    quantize = description['quantize']
    normalize = description['normalize']
    index_type = description['index_type']

    # loading model
    model = load_model(model_name)

    # loading index
    print('loading graph... ', end='', flush=True)
    index = Index(args.indir, index_type, dim, normalize)
    print('done', flush=True)

    # loading links
    print('loading links... ', end='', flush=True)
    with open(os.path.join(args.indir, 'meta.json'), 'r') as f:
        meta_info = json.load(f)
    print('done', flush=True)

    while True:
        search_text = input('Enter search text: ')
        start_time = time.perf_counter()
        if not search_text:
            break
        search_feature = model(search_text)
        if normalize:
            search_feature = l2_normalize(search_feature)
        if quantize:
            search_feature = quantize_data(search_feature, max_val=0.4)
        indices, diffs = index.search_query(search_feature, k=200)
        result_entries = []
        for i, d in zip(indices[0], diffs[0]):
            meta_entry = meta_info[i]
            result_entries.append(ResultEntry(meta_entry['title'], meta_entry['link'], d, meta_entry['views']))

        result_entries.sort(key=lambda en: en.sort_key())
        end_time = time.perf_counter()

        table = Table(('Title', 'Link', 'Views', 'Distance', 'Value'))
        for e in result_entries[:20]:
            table.line(title=e.title, link=e.link, views=e.views, distance=int(e.distance), value=e.sort_key())
        print(table)
        print('results in {:.3f}s\n'.format(end_time - start_time), flush=True)


if __name__ == '__main__':
    main()
