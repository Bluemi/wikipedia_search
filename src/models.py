from typing import List

import numpy as np
import torch
from transformers import AutoTokenizer, AutoModel


class ModelPipeline:
    def __init__(self, tokenizer, model):
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        self.tokenizer = tokenizer
        self.model = model.to(self.device)

    @staticmethod
    def create_e5_base_sts_en_de():
        tokenizer = AutoTokenizer.from_pretrained("danielheinz/e5-base-sts-en-de")
        model = AutoModel.from_pretrained("danielheinz/e5-base-sts-en-de")
        return ModelPipeline(tokenizer, model)

    @staticmethod
    def create_jina_embeddings_v3():
        tokenizer = AutoTokenizer.from_pretrained("jinaai/jina-embeddings-v3")
        model = AutoModel.from_pretrained("jinaai/jina-embeddings-v3", trust_remote_code=True)
        return ModelPipeline(tokenizer, model)

    def __call__(self, texts: List[str]):
        tokens = self.tokenizer(texts, padding=True, truncation=True, return_tensors="pt")
        tokens = {key: val.to(self.device) for key, val in tokens.items()}
        result = self.model(**tokens)
        result = result.last_hidden_state.detach().mean(dim=1)
        result = result.to(torch.float32)
        return result.cpu().numpy()


def l2_normalize(arr):
    return arr / np.linalg.norm(arr, axis=1, keepdims=True)
