import bz2
import re
from typing import Generator

from tqdm import tqdm
import mwxml
import wikitextparser as wtp

DATA_PATH = 'data/dewiki-latest-pages-articles-multistream1.xml-p1p297012.bz2'


def is_section_title(part: str):
    section_pattern = r"^=+ .+? =+$"
    return bool(re.match(section_pattern, part.strip()))


def remove_equals_from_title(part):
    # Define the pattern for section titles and capture the text inside
    section_pattern = r"^=+ (.+?) =+$"
    return re.sub(section_pattern, r"\1", part.strip())


def get_link(title, section) -> str:
    title = title.replace(' ', '_')
    if section is None:
        return f'https://de.wikipedia.org/wiki/{title}'
    section = section.replace(' ', '_')
    return f'https://de.wikipedia.org/wiki/{title}#{section}'


class SectionHeader:
    def __init__(self, title: str, link: str):
        self.title = title
        self.link = link


class TextPart:
    def __init__(self, content: str, header: SectionHeader, link: str):
        self.content = content
        self.header = header
        self.link = link


def process_page(title: str, text: str) -> Generator[TextPart | SectionHeader, None, None]:
    current_header = None

    for part in text.split('\n'):
        part = part.strip()
        if is_section_title(part):
            normed_part = remove_equals_from_title(part)
            current_header = SectionHeader(normed_part, get_link(title, normed_part))
            yield current_header
        elif part not in ('', '*'):
            link = current_header.link if current_header is not None else get_link(title, None)
            yield TextPart(part, current_header, link)


def main():
    with bz2.open(DATA_PATH, 'rt') as f:
        dump = mwxml.Dump.from_file(f)
        counter = 0
        for site_index, site in enumerate(tqdm(dump)):
            revision = next(site)
            # print('#'*80)
            print(site.title)
            for part in process_page(site.title, wtp.parse(revision.text).plain_text()):
                counter += 1
                # if isinstance(part, TextPart):
                    # print(part.content)
                    # print(part.link)
                # elif isinstance(part, SectionHeader):
                    # print('#', part.title)
            # if site_index == 7:
            #     break
    print(counter)


if __name__ == '__main__':
    main()
