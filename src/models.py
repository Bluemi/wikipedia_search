import os
from typing import List

import torch
import open_clip
from transformers import AutoTokenizer, AutoModel


class ModelPipeline:
    def __init__(self, name, tokenizer, model):
        self.name = name
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        self.tokenizer = tokenizer
        self.model = model.to(self.device)

    @staticmethod
    def create_e5_base_sts_en_de():
        tokenizer = AutoTokenizer.from_pretrained("danielheinz/e5-base-sts-en-de")
        model = AutoModel.from_pretrained("danielheinz/e5-base-sts-en-de")
        return ModelPipeline('danielheinz/e5-base-sts-en-de', tokenizer, model)

    @staticmethod
    def create_jina_embeddings_v3():
        tokenizer = AutoTokenizer.from_pretrained("jinaai/jina-embeddings-v3")
        model = AutoModel.from_pretrained("jinaai/jina-embeddings-v3", trust_remote_code=True)
        return ModelPipeline('jinaai/jina-embeddings-v3', tokenizer, model)

    @staticmethod
    def create_jina_clip_v2():
        tokenizer = AutoTokenizer.from_pretrained("jinaai/jina-clip-v2")
        model = AutoModel.from_pretrained("jinaai/jina-clip-v2", trust_remote_code=True)
        return ModelPipeline('jinaai/jina-clip-v2', tokenizer, model)

    @staticmethod
    def create_mcip_vit_l14():
        model_path = os.environ.get('MCIP_VIT_L14_PATH')
        if model_path is None:
            raise ValueError('Environment variable MCIP_VIT_L14_PATH is not set')
        name = "ViT-L-14-336"
        model, _, transform = open_clip.create_model_and_transforms(name, pretrained="openai")

        tokenizer = open_clip.get_tokenizer(name)

        mcip_state_dict = torch.load(model_path)
        model.load_state_dict(mcip_state_dict, strict=True)

        return ModelPipeline(name, tokenizer, model)

    def __call__(self, texts: List[str]):
        if self.name == 'jinaai/jina-clip-v2':
            return self.model.encode_text(texts)
        if self.name == 'ViT-L-14-336':
            tokens = self.tokenizer(texts).to(self.device)
            with torch.no_grad():
                return self.model.encode_text(tokens).cpu().numpy()
        tokens = self.tokenizer(texts, padding=True, truncation=True, return_tensors="pt")
        tokens = {key: val.to(self.device) for key, val in tokens.items()}
        result = self.model(**tokens)
        result = result.last_hidden_state.detach().mean(dim=1)
        result = result.to(torch.float32)
        return result.cpu().numpy()



