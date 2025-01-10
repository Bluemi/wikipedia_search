import bz2
import wiki_parser

import mwxml
from tqdm import tqdm

DATA_PATH = 'data/dewiki-latest-pages-articles-multistream1.xml-p1p297012.bz2'


def get_link(title, section) -> str:
    title = title.replace(' ', '_')
    if section is None:
        return f'https://de.wikipedia.org/wiki/{title}'
    section = section.replace(' ', '_')
    return f'https://de.wikipedia.org/wiki/{title}#{section}'


def main():
    sentence_counter = 0
    with bz2.open(DATA_PATH, 'rt') as f:
        dump = mwxml.Dump.from_file(f)
        for site_index, site in tqdm(enumerate(dump)):
            revision = next(site)
            result = wiki_parser.parse_wiki(revision.text)
            sentence_counter += len(result)

    print('{} parts found'.format(sentence_counter))


if __name__ == '__main__':
    main()
