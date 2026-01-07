"""Weighting helpers for price histories."""

from __future__ import annotations

from typing import Iterable, List


def linear_fade_in_weights(length: int, fade_in: int) -> List[float]:
    if length <= 0:
        raise ValueError("length must be positive")
    if fade_in <= 0:
        raise ValueError("fade_in must be positive")
    weights = []
    for index in range(length):
        weight = min(1.0, (index + 1) / fade_in)
        weights.append(weight)
    return weights


def blend_histories(
    sol_history: Iterable[float],
    bmv_history: Iterable[float],
    fade_in: int,
) -> List[float]:
    sol_list = list(sol_history)
    bmv_list = list(bmv_history)
    if len(sol_list) != len(bmv_list):
        raise ValueError("histories must be the same length")
    weights = linear_fade_in_weights(len(sol_list), fade_in)
    blended = []
    for sol_value, bmv_value, weight in zip(sol_list, bmv_list, weights):
        blended.append(sol_value * weight + bmv_value * (1 - weight))
    return blended
