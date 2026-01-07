import pytest

from solana_dex_bmv.weights import blend_histories, linear_fade_in_weights


def test_linear_fade_in_weights():
    assert linear_fade_in_weights(4, 4) == pytest.approx([0.25, 0.5, 0.75, 1.0])


def test_blend_histories_linear_fade_in():
    sol_history = [10, 20, 30, 40]
    bmv_history = [8, 12, 16, 20]
    blended = blend_histories(sol_history, bmv_history, fade_in=2)
    assert blended == pytest.approx([9.0, 20.0, 30.0, 40.0])
