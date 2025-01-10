import argparse
import os

import deglib


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('indir', type=str)
    return parser.parse_args()


def main():
    args = parse_args()
    print('loading model... ', end='', flush=True)
    from sentence_transformers import SentenceTransformer
    model = SentenceTransformer("svalabs/bi-electra-ms-marco-german-uncased")
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
        search_feature = model.encode(search_text)
        indices, diffs = graph.search(search_feature, 0.2, 10)
        indices = indices[0]
        for i in indices:
            print(links[i])


if __name__ == '__main__':
    main()
