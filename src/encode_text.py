import argparse
import bz2
import os

import wiki_parser

import mwxml
from tqdm import tqdm

from models import ModelPipeline


DEFAULT_DATA_PATH = 'data/input/dewiki-latest-pages-articles-multistream1.xml-p1p297012.bz2'
BATCH_SIZE = 64
MIN_WORDS_PER_PART = 20
MODEL = 'e5_base'


def get_link(title, section) -> str:
    title = title.replace(' ', '_')
    if section is None:
        return f'https://de.wikipedia.org/wiki/{title}'
    section = section.replace(' ', '_')
    return f'https://de.wikipedia.org/wiki/{title}#{section}'


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('outdir', type=str)
    parser.add_argument('--dry', '-d', action='store_true')
    parser.add_argument('-n', type=int, default=0)
    parser.add_argument('--data-path', type=str, default=DEFAULT_DATA_PATH)
    return parser.parse_args()


def main():
    args = parse_args()
    if args.dry:
        model = None
    else:
        print('loading model... ', end='', flush=True)
        if MODEL == 'svalabs':
            from sentence_transformers import SentenceTransformer
            model = SentenceTransformer("svalabs/bi-electra-ms-marco-german-uncased")
        elif MODEL == 'jinaai':
            from transformers import pipeline
            model = pipeline("feature-extraction", model="jinaai/jina-embeddings-v3", trust_remote_code=True)
        elif MODEL == 'e5_base':
            model = ModelPipeline.create_e5_base_sts_en_de()
        else:
            raise ValueError('Unknown model: {}'.format(MODEL))
        print('done', flush=True)

    links = []
    all_features = []
    current_batch = []
    with bz2.open(args.data_path, 'rt') as f:
        dump = mwxml.Dump.from_file(f)
        for site_index, site in tqdm(enumerate(dump)):
            if args.n and site_index == args.n:
                break
            current_title = site.title

            # those articles discuss remove candidates
            if 'Löschkandidaten' in current_title:
                continue
            current_link = get_link(current_title, None)
            revision = next(site)
            result = wiki_parser.parse_wiki(revision.text)
            for heading, part in result:
                if heading == 1:
                    current_link = get_link(current_title, part)
                else:
                    if len(part.split()) > MIN_WORDS_PER_PART:
                        links.append(current_link)
                        current_batch.append(part)
                        if len(current_batch) >= BATCH_SIZE:
                            extract_features(all_features, current_batch, model)
                            current_batch = []

    extract_features(all_features, current_batch, model)

    if not args.dry:
        output_file = os.path.join(args.outdir, 'features.bin')
        dump_vectors_to_binary(output_file, all_features)
        print(f'Features saved to {output_file}')

    link_file = os.path.join(args.outdir, 'links.txt')
    with open(link_file, 'w') as f:
        f.write('\n'.join(links))

    print('num links={}  num_features={}'.format(len(links), sum(f.shape[0] for f in all_features)))


def extract_features(all_features, current_batch, model):
    if model is not None:
        features = model(current_batch)
        all_features.append(features)


def dump_vectors_to_binary(filename, vector_list):
    """
    Dumps a list of numpy vectors to a binary file.

    Args:
        filename (str): The output filename to save the vectors.
        vector_list (list of numpy.ndarray): List of numpy arrays to save.
    """
    with open(filename, 'wb') as f:
        for vector in vector_list:
            data = vector.tobytes()
            f.write(data)


if __name__ == '__main__':
    main()
