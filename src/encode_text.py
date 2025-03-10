import argparse
import bz2
import json
import os
from dataclasses import dataclass
from typing import Iterator, List

import blingfire

from tqdm import tqdm

from models import load_model, get_models
from utils import load_title_to_page_info, normalize_title

BATCH_SIZE = 256
MIN_WORDS_PER_PART = 20
DEFAULT_MODEL = 'jina_clip'


def get_link(title, section=None) -> str:
    title = title.replace(' ', '_')
    if section is None:
        return f'https://de.wikipedia.org/wiki/{title}'
    section = section.replace(' ', '_')
    return f'https://de.wikipedia.org/wiki/{title}#{section}'


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('data', type=str)
    parser.add_argument('outdir', type=str)
    parser.add_argument('--model', type=str, choices=list(get_models().keys()), default='jina_clip')
    parser.add_argument('--dry', '-d', action='store_true')
    parser.add_argument('-n', type=int, default=0)
    return parser.parse_args()


# noinspection PyUnresolvedReferences
def encode_dump_file():
    import mwxml
    import wiki_parser
    args = parse_args()
    model = None
    if not args.dry:
        model = load_model(args.model)

    links = []
    all_features = []
    current_batch = []
    with bz2.open(args.data, 'rt') as f:
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


def count_articles(data_path):
    return sum(1 for _ in tqdm(iterate_summary_files(data_path, fast=True), desc='counting articles'))


def encode_summaries():
    args = parse_args()

    if not os.path.exists(args.outdir):
        os.makedirs(args.outdir)

    model = None
    if not args.dry:
        model = load_model(args.model)

    n_articles = count_articles(args.data)

    title_to_views = load_title_to_page_info()

    all_features = []
    current_batch = []
    meta_info = []
    for index, article in enumerate(tqdm(iterate_summary_files(args.data), total=n_articles)):
        if args.n and index == args.n:
            break
        link = get_link(article.title)

        page_id = -1
        views = 0
        normed_title = normalize_title(article.title)
        if normed_title in title_to_views:
            page_id = title_to_views[normed_title].page_id
            views = title_to_views[normed_title].views

        # add title
        meta_info.append({'link': link, 'title': article.title, 'page_id': page_id, 'views': views})
        current_batch = add_to_batch(article.title, current_batch, model, all_features)

        # add summary
        if article.summary:
            meta_info.append({'link': link, 'title': article.title, 'page_id': page_id, 'views': views})
            current_batch = add_to_batch(article.summary[0], current_batch, model, all_features)

    extract_features(all_features, current_batch, model)

    if not args.dry:
        dump_results(args.outdir, all_features, meta_info, args.model)

    print('num links={}  num_features={}'.format(len(meta_info), sum(f.shape[0] for f in all_features)))


def dump_results(outdir, all_features, meta_info, model):
    output_file = os.path.join(outdir, 'features.bin')
    dump_vectors_to_binary(output_file, all_features)
    print(f'Features saved to {output_file}')

    meta_file = os.path.join(outdir, 'meta.json')
    with open(meta_file, 'w') as f:
        json.dump(meta_info, f)

    description = {
        'dim': all_features[0].shape[1],
        'num_samples': len(meta_info),
        'model': model,
    }
    with open(os.path.join(outdir, 'description.json'), 'w') as f:
        json.dump(description, f, indent=2)


def add_to_batch(text, current_batch, model, all_features):
    current_batch.append(text)
    if len(current_batch) >= BATCH_SIZE:
        extract_features(all_features, current_batch, model)
        current_batch = []
    return current_batch


@dataclass
class ArticleSummary:
    title: str
    summary: List[str]


def iterate_summary_files(summary_dir, fast=False) -> Iterator[ArticleSummary]:
    for input_file in sorted(os.listdir(summary_dir)):
        if not input_file.endswith('.txt'):
            continue
        input_file = os.path.join(summary_dir, input_file)

        with open(input_file, 'r') as f:
            content = f.readlines()

        current_title = None
        current_content = []
        for line in content:
            line: str = line.strip()
            if not line:
                continue

            if line.startswith('[[') and line.endswith(']]'):
                if current_title is not None:
                    yield ArticleSummary(current_title, current_content)
                    current_content = []
                current_title = line[2:-2]
            else:
                if fast:
                    current_content.append(line)
                else:
                    sentences = blingfire.text_to_sentences(line).split('\n')
                    current_content.extend(sentences)

        if current_content:
            yield ArticleSummary(current_title, current_content)


def extract_features(all_features, batch, model):
    if batch:
        if model is not None:
            features = model(batch)
            all_features.append(features)


def dump_vectors_to_binary(filename, vector_list):
    """
    Dumps a list of numpy vectors to a binary file.

    Args:
        filename (str): The output filename to save the vectors.
        vector_list (list of numpy.ndarray): List of numpy arrays to save.
    """
    print(vector_list[0].shape, vector_list[0].dtype)
    with open(filename, 'wb') as f:
        for i, vector in enumerate(vector_list):
            data = vector.tobytes()
            f.write(data)


if __name__ == '__main__':
    # encode_dump_file()
    encode_summaries()
