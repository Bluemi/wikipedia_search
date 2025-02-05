import dataclasses
from typing import List, Dict

import numpy as np
from tqdm import tqdm

INVALID_TITLES = '-_#'


def l2_normalize(arr):
    return arr / np.linalg.norm(arr, axis=1, keepdims=True)


def quantize_data(data, max_val: float = 4.0):
    quantized = np.clip((data + max_val) / (max_val * 2) * 255, 0, 255)
    return np.round(quantized).astype(np.uint8)


@dataclasses.dataclass
class PageInfo:
    page_id: int
    titles: List[str]
    views: int


def normalize_title(title):
    if title in INVALID_TITLES:
        return None
    if title.startswith('"') and title.endswith('"'):
        title = title[1:-1]
        title = title.replace('\\"', '')
    if title == '':
        return None
    title = title.lower().replace(' ', '_')
    return title


def load_page_views() -> Dict[int, PageInfo]:
    page_id_to_info = {}
    with open('data/input/misc/pageviews-202501-user-de') as f:
        for i, line in enumerate(tqdm(f, desc='loading page views')):
            line = line.strip()
            _, title, page_id, _, total_views, _ = line.split(' ')
            total_views = int(total_views)
            if page_id == 'null':
                continue
            page_id = int(page_id)
            if title.startswith('Diskussion:'):
                continue
            title = normalize_title(title)

            if page_id in page_id_to_info:
                page_id_to_info[page_id].views += total_views
                if title is not None and title not in page_id_to_info[page_id].titles:
                    page_id_to_info[page_id].titles.append(title)
            else:
                titles = []
                if title is not None:
                    assert title != ''
                    titles.append(title)
                page_id_to_info[page_id] = PageInfo(page_id, titles, total_views)

    return page_id_to_info


def load_title_to_page_info():
    page_id_to_info = load_page_views()

    title_to_info = {}
    for page_id, page_info in page_id_to_info.items():
        for title in page_info.titles:
            title_to_info[title] = page_info

    return title_to_info
