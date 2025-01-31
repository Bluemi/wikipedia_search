from transformers import AutoModel, AutoTokenizer
import torch

from utils import get_docs, get_queries, show_summary

DEVICE = 'cuda'


def main():
    # Load model and tokenizer
    model_name = 'jinaai/jina-embeddings-v3'
    tokenizer = AutoTokenizer.from_pretrained(model_name, trust_remote_code=True)
    model = AutoModel.from_pretrained(model_name, trust_remote_code=True).to(DEVICE)

    # Example queries and passages
    queries = get_queries()
    docs = get_docs()

    print('encoding queries')
    query_embeddings = encode_text(get_queries(), model, tokenizer)

    print('encoding docs')
    doc_embeddings = encode_text(get_docs(), model, tokenizer)

    print(query_embeddings.shape, doc_embeddings.shape)

    show_summary(queries, query_embeddings, docs, doc_embeddings)


# Encode text with task specification
def encode_text(texts, model, tokenizer):
    tokens = tokenizer(texts, padding=True, truncation=True, return_tensors="pt", max_length=512)
    # inputs["task"] = task  # Specify task type
    tokens = {key: val.to(DEVICE) for key, val in tokens.items()}
    with torch.no_grad():
        outputs = model(**tokens)
    return outputs.last_hidden_state[:, 0, :].cpu().to(torch.float32).numpy()  # Use CLS token embeddings


if __name__ == '__main__':
    main()
