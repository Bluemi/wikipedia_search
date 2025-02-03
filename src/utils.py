import numpy as np


def l2_normalize(arr):
    return arr / np.linalg.norm(arr, axis=1, keepdims=True)
