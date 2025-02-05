import numpy as np


def l2_normalize(arr):
    return arr / np.linalg.norm(arr, axis=1, keepdims=True)


def quantize_data(data, max_val: float = 4.0):
    quantized = np.clip((data + max_val) / (max_val * 2) * 255, 0, 255)
    return np.round(quantized).astype(np.uint8)

