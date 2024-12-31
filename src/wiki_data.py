import bz2
import re
import time
from typing import Generator

from tqdm import tqdm
import mwxml
import wikitextparser as wtp

from tables import Table

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
    table = Table(('Title', 'Site', 'Parse', 'Process'), float_precision=4)
    with bz2.open(DATA_PATH, 'rt') as f:
        dump = mwxml.Dump.from_file(f)
        counter = 0
        sum_site = 0
        sum_parse = 0
        sum_process = 0
        start_time = time.perf_counter()
        for site_index, site in enumerate(dump):
            revision = next(site)
            end_time = time.perf_counter()
            site_duration = end_time - start_time
            sum_site += site_duration

            start_time = time.perf_counter()
            parsed = wtp.parse(revision.text).plain_text()
            end_time = time.perf_counter()
            parse_duration = end_time - start_time
            sum_parse += parse_duration

            start_time = time.perf_counter()
            for part in process_page(site.title, parsed):
                counter += 1
                # if isinstance(part, TextPart):
                    # print(part.content)
                    # print(part.link)
                # elif isinstance(part, SectionHeader):
                    # print('#', part.title)
            end_time = time.perf_counter()
            process_duration = end_time - start_time
            sum_process += process_duration

            table.line(title=site.title, site=site_duration, parse=parse_duration, process=process_duration)

            if site_index == 2000:
                break
            start_time = time.perf_counter()

    table.line(title='SUM', site=sum_site, parse=sum_parse, process=sum_process)
    print(table)


if __name__ == '__main__':
    main()
