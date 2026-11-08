# Detector evaluation

Chosen config: `{"ctx_delta": 0.0, "dictionary_bonus": 0.0, "final_threshold": 2.0, "max_check_len": 3, "min_len": 2, "neutral_rule": true, "orthography_convert": 1.8, "orthography_keep": 1.8, "threshold": 4.0}`

## Dev tuning grid

| typing threshold | final threshold | ctx delta | min len | max check len | neutral rule | max FP/1000 | recall % | meets FP cap |
|---:|---:|---:|---:|---:|---|---:|---:|---|
| 0 | 0 | 0 | 2 | 3 | off | 32.998 | 99.51 | no |
| 0 | 0 | 0 | 2 | 3 | on | 32.998 | 99.61 | no |
| 0 | 0 | 0 | 2 | 4 | off | 32.998 | 99.51 | no |
| 0 | 0 | 0 | 2 | 4 | on | 32.998 | 99.61 | no |
| 0 | 0 | 0 | 2 | 6 | off | 32.998 | 99.51 | no |
| 0 | 0 | 0 | 2 | 6 | on | 32.998 | 99.61 | no |
| 0 | 0 | 0 | 2 | 8 | off | 32.998 | 99.51 | no |
| 0 | 0 | 0 | 2 | 8 | on | 32.998 | 99.61 | no |
| 0 | 0 | 0 | 2 | None | off | 32.998 | 99.51 | no |
| 0 | 0 | 0 | 2 | None | on | 32.998 | 99.61 | no |
| 0 | 0 | 0 | 3 | 3 | off | 3.313 | 97.21 | no |
| 0 | 0 | 0 | 3 | 3 | on | 3.313 | 97.30 | no |
| 0 | 0 | 0 | 3 | 4 | off | 3.976 | 97.21 | no |
| 0 | 0 | 0 | 3 | 4 | on | 3.976 | 97.30 | no |
| 0 | 0 | 0 | 3 | 6 | off | 4.639 | 97.21 | no |
| 0 | 0 | 0 | 3 | 6 | on | 4.639 | 97.30 | no |
| 0 | 0 | 0 | 3 | 8 | off | 4.639 | 97.21 | no |
| 0 | 0 | 0 | 3 | 8 | on | 4.639 | 97.30 | no |
| 0 | 0 | 0 | 3 | None | off | 4.639 | 97.21 | no |
| 0 | 0 | 0 | 3 | None | on | 4.639 | 97.30 | no |
| 0 | 0 | 0.25 | 2 | 3 | off | 39.150 | 99.51 | no |
| 0 | 0 | 0.25 | 2 | 3 | on | 39.150 | 99.61 | no |
| 0 | 0 | 0.25 | 2 | 4 | off | 39.150 | 99.51 | no |
| 0 | 0 | 0.25 | 2 | 4 | on | 39.150 | 99.61 | no |
| 0 | 0 | 0.25 | 2 | 6 | off | 39.150 | 99.51 | no |
| 0 | 0 | 0.25 | 2 | 6 | on | 39.150 | 99.61 | no |
| 0 | 0 | 0.25 | 2 | 8 | off | 39.150 | 99.51 | no |
| 0 | 0 | 0.25 | 2 | 8 | on | 39.150 | 99.61 | no |
| 0 | 0 | 0.25 | 2 | None | off | 39.150 | 99.51 | no |
| 0 | 0 | 0.25 | 2 | None | on | 39.150 | 99.61 | no |
| 0 | 0 | 0.25 | 3 | 3 | off | 2.796 | 97.21 | no |
| 0 | 0 | 0.25 | 3 | 3 | on | 2.796 | 97.30 | no |
| 0 | 0 | 0.25 | 3 | 4 | off | 2.796 | 97.21 | no |
| 0 | 0 | 0.25 | 3 | 4 | on | 2.796 | 97.30 | no |
| 0 | 0 | 0.25 | 3 | 6 | off | 3.976 | 97.21 | no |
| 0 | 0 | 0.25 | 3 | 6 | on | 3.976 | 97.30 | no |
| 0 | 0 | 0.25 | 3 | 8 | off | 3.976 | 97.21 | no |
| 0 | 0 | 0.25 | 3 | 8 | on | 3.976 | 97.30 | no |
| 0 | 0 | 0.25 | 3 | None | off | 3.976 | 97.21 | no |
| 0 | 0 | 0.25 | 3 | None | on | 3.976 | 97.30 | no |
| 0 | 2 | 0 | 2 | 3 | off | 32.998 | 99.51 | no |
| 0 | 2 | 0 | 2 | 3 | on | 32.998 | 99.61 | no |
| 0 | 2 | 0 | 2 | 4 | off | 32.998 | 99.51 | no |
| 0 | 2 | 0 | 2 | 4 | on | 32.998 | 99.61 | no |
| 0 | 2 | 0 | 2 | 6 | off | 32.998 | 99.51 | no |
| 0 | 2 | 0 | 2 | 6 | on | 32.998 | 99.61 | no |
| 0 | 2 | 0 | 2 | 8 | off | 32.998 | 99.51 | no |
| 0 | 2 | 0 | 2 | 8 | on | 32.998 | 99.61 | no |
| 0 | 2 | 0 | 2 | None | off | 32.998 | 99.51 | no |
| 0 | 2 | 0 | 2 | None | on | 32.998 | 99.61 | no |
| 0 | 2 | 0 | 3 | 3 | off | 3.313 | 97.18 | no |
| 0 | 2 | 0 | 3 | 3 | on | 3.313 | 97.27 | no |
| 0 | 2 | 0 | 3 | 4 | off | 3.976 | 97.18 | no |
| 0 | 2 | 0 | 3 | 4 | on | 3.976 | 97.27 | no |
| 0 | 2 | 0 | 3 | 6 | off | 4.639 | 97.18 | no |
| 0 | 2 | 0 | 3 | 6 | on | 4.639 | 97.27 | no |
| 0 | 2 | 0 | 3 | 8 | off | 4.639 | 97.18 | no |
| 0 | 2 | 0 | 3 | 8 | on | 4.639 | 97.27 | no |
| 0 | 2 | 0 | 3 | None | off | 4.639 | 97.18 | no |
| 0 | 2 | 0 | 3 | None | on | 4.639 | 97.27 | no |
| 0 | 2 | 0.25 | 2 | 3 | off | 39.150 | 99.51 | no |
| 0 | 2 | 0.25 | 2 | 3 | on | 39.150 | 99.61 | no |
| 0 | 2 | 0.25 | 2 | 4 | off | 39.150 | 99.51 | no |
| 0 | 2 | 0.25 | 2 | 4 | on | 39.150 | 99.61 | no |
| 0 | 2 | 0.25 | 2 | 6 | off | 39.150 | 99.51 | no |
| 0 | 2 | 0.25 | 2 | 6 | on | 39.150 | 99.61 | no |
| 0 | 2 | 0.25 | 2 | 8 | off | 39.150 | 99.51 | no |
| 0 | 2 | 0.25 | 2 | 8 | on | 39.150 | 99.61 | no |
| 0 | 2 | 0.25 | 2 | None | off | 39.150 | 99.51 | no |
| 0 | 2 | 0.25 | 2 | None | on | 39.150 | 99.61 | no |
| 0 | 2 | 0.25 | 3 | 3 | off | 2.796 | 97.21 | no |
| 0 | 2 | 0.25 | 3 | 3 | on | 2.796 | 97.30 | no |
| 0 | 2 | 0.25 | 3 | 4 | off | 2.796 | 97.21 | no |
| 0 | 2 | 0.25 | 3 | 4 | on | 2.796 | 97.30 | no |
| 0 | 2 | 0.25 | 3 | 6 | off | 3.976 | 97.21 | no |
| 0 | 2 | 0.25 | 3 | 6 | on | 3.976 | 97.30 | no |
| 0 | 2 | 0.25 | 3 | 8 | off | 3.976 | 97.21 | no |
| 0 | 2 | 0.25 | 3 | 8 | on | 3.976 | 97.30 | no |
| 0 | 2 | 0.25 | 3 | None | off | 3.976 | 97.21 | no |
| 0 | 2 | 0.25 | 3 | None | on | 3.976 | 97.30 | no |
| 0 | 4 | 0 | 2 | 3 | off | 32.998 | 99.45 | no |
| 0 | 4 | 0 | 2 | 3 | on | 32.998 | 99.55 | no |
| 0 | 4 | 0 | 2 | 4 | off | 32.998 | 99.48 | no |
| 0 | 4 | 0 | 2 | 4 | on | 32.998 | 99.58 | no |
| 0 | 4 | 0 | 2 | 6 | off | 32.998 | 99.48 | no |
| 0 | 4 | 0 | 2 | 6 | on | 32.998 | 99.58 | no |
| 0 | 4 | 0 | 2 | 8 | off | 32.998 | 99.48 | no |
| 0 | 4 | 0 | 2 | 8 | on | 32.998 | 99.58 | no |
| 0 | 4 | 0 | 2 | None | off | 32.998 | 99.48 | no |
| 0 | 4 | 0 | 2 | None | on | 32.998 | 99.58 | no |
| 0 | 4 | 0 | 3 | 3 | off | 3.313 | 97.12 | no |
| 0 | 4 | 0 | 3 | 3 | on | 3.313 | 97.21 | no |
| 0 | 4 | 0 | 3 | 4 | off | 3.976 | 97.18 | no |
| 0 | 4 | 0 | 3 | 4 | on | 3.976 | 97.27 | no |
| 0 | 4 | 0 | 3 | 6 | off | 4.639 | 97.18 | no |
| 0 | 4 | 0 | 3 | 6 | on | 4.639 | 97.27 | no |
| 0 | 4 | 0 | 3 | 8 | off | 4.639 | 97.18 | no |
| 0 | 4 | 0 | 3 | 8 | on | 4.639 | 97.27 | no |
| 0 | 4 | 0 | 3 | None | off | 4.639 | 97.18 | no |
| 0 | 4 | 0 | 3 | None | on | 4.639 | 97.27 | no |
| 0 | 4 | 0.25 | 2 | 3 | off | 39.150 | 99.45 | no |
| 0 | 4 | 0.25 | 2 | 3 | on | 39.150 | 99.55 | no |
| 0 | 4 | 0.25 | 2 | 4 | off | 39.150 | 99.48 | no |
| 0 | 4 | 0.25 | 2 | 4 | on | 39.150 | 99.58 | no |
| 0 | 4 | 0.25 | 2 | 6 | off | 39.150 | 99.48 | no |
| 0 | 4 | 0.25 | 2 | 6 | on | 39.150 | 99.58 | no |
| 0 | 4 | 0.25 | 2 | 8 | off | 39.150 | 99.48 | no |
| 0 | 4 | 0.25 | 2 | 8 | on | 39.150 | 99.58 | no |
| 0 | 4 | 0.25 | 2 | None | off | 39.150 | 99.48 | no |
| 0 | 4 | 0.25 | 2 | None | on | 39.150 | 99.58 | no |
| 0 | 4 | 0.25 | 3 | 3 | off | 2.796 | 97.18 | no |
| 0 | 4 | 0.25 | 3 | 3 | on | 2.796 | 97.27 | no |
| 0 | 4 | 0.25 | 3 | 4 | off | 2.796 | 97.21 | no |
| 0 | 4 | 0.25 | 3 | 4 | on | 2.796 | 97.30 | no |
| 0 | 4 | 0.25 | 3 | 6 | off | 3.976 | 97.21 | no |
| 0 | 4 | 0.25 | 3 | 6 | on | 3.976 | 97.30 | no |
| 0 | 4 | 0.25 | 3 | 8 | off | 3.976 | 97.21 | no |
| 0 | 4 | 0.25 | 3 | 8 | on | 3.976 | 97.30 | no |
| 0 | 4 | 0.25 | 3 | None | off | 3.976 | 97.21 | no |
| 0 | 4 | 0.25 | 3 | None | on | 3.976 | 97.30 | no |
| 2 | 0 | 0 | 2 | 3 | off | 1.325 | 99.51 | no |
| 2 | 0 | 0 | 2 | 3 | on | 1.325 | 99.61 | no |
| 2 | 0 | 0 | 2 | 4 | off | 2.651 | 99.51 | no |
| 2 | 0 | 0 | 2 | 4 | on | 2.651 | 99.61 | no |
| 2 | 0 | 0 | 2 | 6 | off | 2.651 | 99.51 | no |
| 2 | 0 | 0 | 2 | 6 | on | 2.651 | 99.61 | no |
| 2 | 0 | 0 | 2 | 8 | off | 2.651 | 99.51 | no |
| 2 | 0 | 0 | 2 | 8 | on | 2.651 | 99.61 | no |
| 2 | 0 | 0 | 2 | None | off | 2.651 | 99.51 | no |
| 2 | 0 | 0 | 2 | None | on | 2.651 | 99.61 | no |
| 2 | 0 | 0 | 3 | 3 | off | 0.663 | 97.21 | yes |
| 2 | 0 | 0 | 3 | 3 | on | 0.663 | 97.30 | yes |
| 2 | 0 | 0 | 3 | 4 | off | 1.988 | 97.21 | no |
| 2 | 0 | 0 | 3 | 4 | on | 1.988 | 97.30 | no |
| 2 | 0 | 0 | 3 | 6 | off | 1.988 | 97.21 | no |
| 2 | 0 | 0 | 3 | 6 | on | 1.988 | 97.30 | no |
| 2 | 0 | 0 | 3 | 8 | off | 1.988 | 97.21 | no |
| 2 | 0 | 0 | 3 | 8 | on | 1.988 | 97.30 | no |
| 2 | 0 | 0 | 3 | None | off | 1.988 | 97.21 | no |
| 2 | 0 | 0 | 3 | None | on | 1.988 | 97.30 | no |
| 2 | 0 | 0.25 | 2 | 3 | off | 1.325 | 99.51 | no |
| 2 | 0 | 0.25 | 2 | 3 | on | 1.325 | 99.61 | no |
| 2 | 0 | 0.25 | 2 | 4 | off | 2.651 | 99.51 | no |
| 2 | 0 | 0.25 | 2 | 4 | on | 2.651 | 99.61 | no |
| 2 | 0 | 0.25 | 2 | 6 | off | 2.651 | 99.51 | no |
| 2 | 0 | 0.25 | 2 | 6 | on | 2.651 | 99.61 | no |
| 2 | 0 | 0.25 | 2 | 8 | off | 2.651 | 99.51 | no |
| 2 | 0 | 0.25 | 2 | 8 | on | 2.651 | 99.61 | no |
| 2 | 0 | 0.25 | 2 | None | off | 2.651 | 99.51 | no |
| 2 | 0 | 0.25 | 2 | None | on | 2.651 | 99.61 | no |
| 2 | 0 | 0.25 | 3 | 3 | off | 0.663 | 97.21 | yes |
| 2 | 0 | 0.25 | 3 | 3 | on | 0.663 | 97.30 | yes |
| 2 | 0 | 0.25 | 3 | 4 | off | 1.988 | 97.21 | no |
| 2 | 0 | 0.25 | 3 | 4 | on | 1.988 | 97.30 | no |
| 2 | 0 | 0.25 | 3 | 6 | off | 1.988 | 97.21 | no |
| 2 | 0 | 0.25 | 3 | 6 | on | 1.988 | 97.30 | no |
| 2 | 0 | 0.25 | 3 | 8 | off | 1.988 | 97.21 | no |
| 2 | 0 | 0.25 | 3 | 8 | on | 1.988 | 97.30 | no |
| 2 | 0 | 0.25 | 3 | None | off | 1.988 | 97.21 | no |
| 2 | 0 | 0.25 | 3 | None | on | 1.988 | 97.30 | no |
| 2 | 2 | 0 | 2 | 3 | off | 1.119 | 99.42 | no |
| 2 | 2 | 0 | 2 | 3 | on | 1.119 | 99.51 | no |
| 2 | 2 | 0 | 2 | 4 | off | 1.988 | 99.42 | no |
| 2 | 2 | 0 | 2 | 4 | on | 1.988 | 99.51 | no |
| 2 | 2 | 0 | 2 | 6 | off | 1.988 | 99.42 | no |
| 2 | 2 | 0 | 2 | 6 | on | 1.988 | 99.51 | no |
| 2 | 2 | 0 | 2 | 8 | off | 1.988 | 99.42 | no |
| 2 | 2 | 0 | 2 | 8 | on | 1.988 | 99.51 | no |
| 2 | 2 | 0 | 2 | None | off | 1.988 | 99.42 | no |
| 2 | 2 | 0 | 2 | None | on | 1.988 | 99.51 | no |
| 2 | 2 | 0 | 3 | 3 | off | 0.663 | 97.09 | yes |
| 2 | 2 | 0 | 3 | 3 | on | 0.663 | 97.18 | yes |
| 2 | 2 | 0 | 3 | 4 | off | 1.988 | 97.09 | no |
| 2 | 2 | 0 | 3 | 4 | on | 1.988 | 97.18 | no |
| 2 | 2 | 0 | 3 | 6 | off | 1.988 | 97.09 | no |
| 2 | 2 | 0 | 3 | 6 | on | 1.988 | 97.18 | no |
| 2 | 2 | 0 | 3 | 8 | off | 1.988 | 97.09 | no |
| 2 | 2 | 0 | 3 | 8 | on | 1.988 | 97.18 | no |
| 2 | 2 | 0 | 3 | None | off | 1.988 | 97.09 | no |
| 2 | 2 | 0 | 3 | None | on | 1.988 | 97.18 | no |
| 2 | 2 | 0.25 | 2 | 3 | off | 1.119 | 99.42 | no |
| 2 | 2 | 0.25 | 2 | 3 | on | 1.119 | 99.51 | no |
| 2 | 2 | 0.25 | 2 | 4 | off | 1.988 | 99.42 | no |
| 2 | 2 | 0.25 | 2 | 4 | on | 1.988 | 99.51 | no |
| 2 | 2 | 0.25 | 2 | 6 | off | 1.988 | 99.42 | no |
| 2 | 2 | 0.25 | 2 | 6 | on | 1.988 | 99.51 | no |
| 2 | 2 | 0.25 | 2 | 8 | off | 1.988 | 99.42 | no |
| 2 | 2 | 0.25 | 2 | 8 | on | 1.988 | 99.51 | no |
| 2 | 2 | 0.25 | 2 | None | off | 1.988 | 99.42 | no |
| 2 | 2 | 0.25 | 2 | None | on | 1.988 | 99.51 | no |
| 2 | 2 | 0.25 | 3 | 3 | off | 0.663 | 97.09 | yes |
| 2 | 2 | 0.25 | 3 | 3 | on | 0.663 | 97.18 | yes |
| 2 | 2 | 0.25 | 3 | 4 | off | 1.988 | 97.09 | no |
| 2 | 2 | 0.25 | 3 | 4 | on | 1.988 | 97.18 | no |
| 2 | 2 | 0.25 | 3 | 6 | off | 1.988 | 97.09 | no |
| 2 | 2 | 0.25 | 3 | 6 | on | 1.988 | 97.18 | no |
| 2 | 2 | 0.25 | 3 | 8 | off | 1.988 | 97.09 | no |
| 2 | 2 | 0.25 | 3 | 8 | on | 1.988 | 97.18 | no |
| 2 | 2 | 0.25 | 3 | None | off | 1.988 | 97.09 | no |
| 2 | 2 | 0.25 | 3 | None | on | 1.988 | 97.18 | no |
| 2 | 4 | 0 | 2 | 3 | off | 1.119 | 99.27 | no |
| 2 | 4 | 0 | 2 | 3 | on | 1.119 | 99.36 | no |
| 2 | 4 | 0 | 2 | 4 | off | 1.988 | 99.27 | no |
| 2 | 4 | 0 | 2 | 4 | on | 1.988 | 99.36 | no |
| 2 | 4 | 0 | 2 | 6 | off | 1.988 | 99.39 | no |
| 2 | 4 | 0 | 2 | 6 | on | 1.988 | 99.48 | no |
| 2 | 4 | 0 | 2 | 8 | off | 1.988 | 99.39 | no |
| 2 | 4 | 0 | 2 | 8 | on | 1.988 | 99.48 | no |
| 2 | 4 | 0 | 2 | None | off | 1.988 | 99.39 | no |
| 2 | 4 | 0 | 2 | None | on | 1.988 | 99.48 | no |
| 2 | 4 | 0 | 3 | 3 | off | 0.663 | 96.97 | yes |
| 2 | 4 | 0 | 3 | 3 | on | 0.663 | 97.06 | yes |
| 2 | 4 | 0 | 3 | 4 | off | 1.988 | 96.97 | no |
| 2 | 4 | 0 | 3 | 4 | on | 1.988 | 97.06 | no |
| 2 | 4 | 0 | 3 | 6 | off | 1.988 | 97.09 | no |
| 2 | 4 | 0 | 3 | 6 | on | 1.988 | 97.18 | no |
| 2 | 4 | 0 | 3 | 8 | off | 1.988 | 97.09 | no |
| 2 | 4 | 0 | 3 | 8 | on | 1.988 | 97.18 | no |
| 2 | 4 | 0 | 3 | None | off | 1.988 | 97.09 | no |
| 2 | 4 | 0 | 3 | None | on | 1.988 | 97.18 | no |
| 2 | 4 | 0.25 | 2 | 3 | off | 1.119 | 99.30 | no |
| 2 | 4 | 0.25 | 2 | 3 | on | 1.119 | 99.39 | no |
| 2 | 4 | 0.25 | 2 | 4 | off | 1.988 | 99.30 | no |
| 2 | 4 | 0.25 | 2 | 4 | on | 1.988 | 99.39 | no |
| 2 | 4 | 0.25 | 2 | 6 | off | 1.988 | 99.39 | no |
| 2 | 4 | 0.25 | 2 | 6 | on | 1.988 | 99.48 | no |
| 2 | 4 | 0.25 | 2 | 8 | off | 1.988 | 99.39 | no |
| 2 | 4 | 0.25 | 2 | 8 | on | 1.988 | 99.48 | no |
| 2 | 4 | 0.25 | 2 | None | off | 1.988 | 99.39 | no |
| 2 | 4 | 0.25 | 2 | None | on | 1.988 | 99.48 | no |
| 2 | 4 | 0.25 | 3 | 3 | off | 0.663 | 97.00 | yes |
| 2 | 4 | 0.25 | 3 | 3 | on | 0.663 | 97.09 | yes |
| 2 | 4 | 0.25 | 3 | 4 | off | 1.988 | 97.00 | no |
| 2 | 4 | 0.25 | 3 | 4 | on | 1.988 | 97.09 | no |
| 2 | 4 | 0.25 | 3 | 6 | off | 1.988 | 97.09 | no |
| 2 | 4 | 0.25 | 3 | 6 | on | 1.988 | 97.18 | no |
| 2 | 4 | 0.25 | 3 | 8 | off | 1.988 | 97.09 | no |
| 2 | 4 | 0.25 | 3 | 8 | on | 1.988 | 97.18 | no |
| 2 | 4 | 0.25 | 3 | None | off | 1.988 | 97.09 | no |
| 2 | 4 | 0.25 | 3 | None | on | 1.988 | 97.18 | no |
| 4 | 0 | 0 | 2 | 3 | off | 1.325 | 99.51 | no |
| 4 | 0 | 0 | 2 | 3 | on | 1.325 | 99.61 | no |
| 4 | 0 | 0 | 2 | 4 | off | 1.325 | 99.51 | no |
| 4 | 0 | 0 | 2 | 4 | on | 1.325 | 99.61 | no |
| 4 | 0 | 0 | 2 | 6 | off | 1.988 | 99.51 | no |
| 4 | 0 | 0 | 2 | 6 | on | 1.988 | 99.61 | no |
| 4 | 0 | 0 | 2 | 8 | off | 1.988 | 99.51 | no |
| 4 | 0 | 0 | 2 | 8 | on | 1.988 | 99.61 | no |
| 4 | 0 | 0 | 2 | None | off | 1.988 | 99.51 | no |
| 4 | 0 | 0 | 2 | None | on | 1.988 | 99.61 | no |
| 4 | 0 | 0 | 3 | 3 | off | 0.663 | 97.21 | yes |
| 4 | 0 | 0 | 3 | 3 | on | 0.663 | 97.30 | yes |
| 4 | 0 | 0 | 3 | 4 | off | 0.663 | 97.21 | yes |
| 4 | 0 | 0 | 3 | 4 | on | 0.663 | 97.30 | yes |
| 4 | 0 | 0 | 3 | 6 | off | 1.325 | 97.21 | no |
| 4 | 0 | 0 | 3 | 6 | on | 1.325 | 97.30 | no |
| 4 | 0 | 0 | 3 | 8 | off | 1.325 | 97.21 | no |
| 4 | 0 | 0 | 3 | 8 | on | 1.325 | 97.30 | no |
| 4 | 0 | 0 | 3 | None | off | 1.325 | 97.21 | no |
| 4 | 0 | 0 | 3 | None | on | 1.325 | 97.30 | no |
| 4 | 0 | 0.25 | 2 | 3 | off | 1.325 | 99.51 | no |
| 4 | 0 | 0.25 | 2 | 3 | on | 1.325 | 99.61 | no |
| 4 | 0 | 0.25 | 2 | 4 | off | 1.325 | 99.51 | no |
| 4 | 0 | 0.25 | 2 | 4 | on | 1.325 | 99.61 | no |
| 4 | 0 | 0.25 | 2 | 6 | off | 1.988 | 99.51 | no |
| 4 | 0 | 0.25 | 2 | 6 | on | 1.988 | 99.61 | no |
| 4 | 0 | 0.25 | 2 | 8 | off | 1.988 | 99.51 | no |
| 4 | 0 | 0.25 | 2 | 8 | on | 1.988 | 99.61 | no |
| 4 | 0 | 0.25 | 2 | None | off | 1.988 | 99.51 | no |
| 4 | 0 | 0.25 | 2 | None | on | 1.988 | 99.61 | no |
| 4 | 0 | 0.25 | 3 | 3 | off | 0.663 | 97.21 | yes |
| 4 | 0 | 0.25 | 3 | 3 | on | 0.663 | 97.30 | yes |
| 4 | 0 | 0.25 | 3 | 4 | off | 0.663 | 97.21 | yes |
| 4 | 0 | 0.25 | 3 | 4 | on | 0.663 | 97.30 | yes |
| 4 | 0 | 0.25 | 3 | 6 | off | 1.325 | 97.21 | no |
| 4 | 0 | 0.25 | 3 | 6 | on | 1.325 | 97.30 | no |
| 4 | 0 | 0.25 | 3 | 8 | off | 1.325 | 97.21 | no |
| 4 | 0 | 0.25 | 3 | 8 | on | 1.325 | 97.30 | no |
| 4 | 0 | 0.25 | 3 | None | off | 1.325 | 97.21 | no |
| 4 | 0 | 0.25 | 3 | None | on | 1.325 | 97.30 | no |
| 4 | 2 | 0 | 2 | 3 | off | 0.663 | 99.39 | yes |
| 4 | 2 | 0 | 2 | 3 | on | 0.663 | 99.48 | yes |
| 4 | 2 | 0 | 2 | 4 | off | 0.663 | 99.39 | yes |
| 4 | 2 | 0 | 2 | 4 | on | 0.663 | 99.48 | yes |
| 4 | 2 | 0 | 2 | 6 | off | 1.325 | 99.39 | no |
| 4 | 2 | 0 | 2 | 6 | on | 1.325 | 99.48 | no |
| 4 | 2 | 0 | 2 | 8 | off | 1.325 | 99.39 | no |
| 4 | 2 | 0 | 2 | 8 | on | 1.325 | 99.48 | no |
| 4 | 2 | 0 | 2 | None | off | 1.325 | 99.39 | no |
| 4 | 2 | 0 | 2 | None | on | 1.325 | 99.48 | no |
| 4 | 2 | 0 | 3 | 3 | off | 0.663 | 97.09 | yes |
| 4 | 2 | 0 | 3 | 3 | on | 0.663 | 97.18 | yes |
| 4 | 2 | 0 | 3 | 4 | off | 0.663 | 97.09 | yes |
| 4 | 2 | 0 | 3 | 4 | on | 0.663 | 97.18 | yes |
| 4 | 2 | 0 | 3 | 6 | off | 1.325 | 97.09 | no |
| 4 | 2 | 0 | 3 | 6 | on | 1.325 | 97.18 | no |
| 4 | 2 | 0 | 3 | 8 | off | 1.325 | 97.09 | no |
| 4 | 2 | 0 | 3 | 8 | on | 1.325 | 97.18 | no |
| 4 | 2 | 0 | 3 | None | off | 1.325 | 97.09 | no |
| 4 | 2 | 0 | 3 | None | on | 1.325 | 97.18 | no |
| 4 | 2 | 0.25 | 2 | 3 | off | 0.663 | 99.39 | yes |
| 4 | 2 | 0.25 | 2 | 3 | on | 0.663 | 99.48 | yes |
| 4 | 2 | 0.25 | 2 | 4 | off | 0.663 | 99.39 | yes |
| 4 | 2 | 0.25 | 2 | 4 | on | 0.663 | 99.48 | yes |
| 4 | 2 | 0.25 | 2 | 6 | off | 1.325 | 99.39 | no |
| 4 | 2 | 0.25 | 2 | 6 | on | 1.325 | 99.48 | no |
| 4 | 2 | 0.25 | 2 | 8 | off | 1.325 | 99.39 | no |
| 4 | 2 | 0.25 | 2 | 8 | on | 1.325 | 99.48 | no |
| 4 | 2 | 0.25 | 2 | None | off | 1.325 | 99.39 | no |
| 4 | 2 | 0.25 | 2 | None | on | 1.325 | 99.48 | no |
| 4 | 2 | 0.25 | 3 | 3 | off | 0.663 | 97.09 | yes |
| 4 | 2 | 0.25 | 3 | 3 | on | 0.663 | 97.18 | yes |
| 4 | 2 | 0.25 | 3 | 4 | off | 0.663 | 97.09 | yes |
| 4 | 2 | 0.25 | 3 | 4 | on | 0.663 | 97.18 | yes |
| 4 | 2 | 0.25 | 3 | 6 | off | 1.325 | 97.09 | no |
| 4 | 2 | 0.25 | 3 | 6 | on | 1.325 | 97.18 | no |
| 4 | 2 | 0.25 | 3 | 8 | off | 1.325 | 97.09 | no |
| 4 | 2 | 0.25 | 3 | 8 | on | 1.325 | 97.18 | no |
| 4 | 2 | 0.25 | 3 | None | off | 1.325 | 97.09 | no |
| 4 | 2 | 0.25 | 3 | None | on | 1.325 | 97.18 | no |
| 4 | 4 | 0 | 2 | 3 | off | 0.663 | 99.18 | yes |
| 4 | 4 | 0 | 2 | 3 | on | 0.663 | 99.27 | yes |
| 4 | 4 | 0 | 2 | 4 | off | 0.663 | 99.18 | yes |
| 4 | 4 | 0 | 2 | 4 | on | 0.663 | 99.27 | yes |
| 4 | 4 | 0 | 2 | 6 | off | 1.325 | 99.18 | no |
| 4 | 4 | 0 | 2 | 6 | on | 1.325 | 99.27 | no |
| 4 | 4 | 0 | 2 | 8 | off | 1.325 | 99.21 | no |
| 4 | 4 | 0 | 2 | 8 | on | 1.325 | 99.30 | no |
| 4 | 4 | 0 | 2 | None | off | 1.325 | 99.21 | no |
| 4 | 4 | 0 | 2 | None | on | 1.325 | 99.30 | no |
| 4 | 4 | 0 | 3 | 3 | off | 0.663 | 96.91 | yes |
| 4 | 4 | 0 | 3 | 3 | on | 0.663 | 97.00 | yes |
| 4 | 4 | 0 | 3 | 4 | off | 0.663 | 96.91 | yes |
| 4 | 4 | 0 | 3 | 4 | on | 0.663 | 97.00 | yes |
| 4 | 4 | 0 | 3 | 6 | off | 1.325 | 96.91 | no |
| 4 | 4 | 0 | 3 | 6 | on | 1.325 | 97.00 | no |
| 4 | 4 | 0 | 3 | 8 | off | 1.325 | 96.94 | no |
| 4 | 4 | 0 | 3 | 8 | on | 1.325 | 97.03 | no |
| 4 | 4 | 0 | 3 | None | off | 1.325 | 96.94 | no |
| 4 | 4 | 0 | 3 | None | on | 1.325 | 97.03 | no |
| 4 | 4 | 0.25 | 2 | 3 | off | 0.663 | 99.21 | yes |
| 4 | 4 | 0.25 | 2 | 3 | on | 0.663 | 99.30 | yes |
| 4 | 4 | 0.25 | 2 | 4 | off | 0.663 | 99.21 | yes |
| 4 | 4 | 0.25 | 2 | 4 | on | 0.663 | 99.30 | yes |
| 4 | 4 | 0.25 | 2 | 6 | off | 1.325 | 99.21 | no |
| 4 | 4 | 0.25 | 2 | 6 | on | 1.325 | 99.30 | no |
| 4 | 4 | 0.25 | 2 | 8 | off | 1.325 | 99.24 | no |
| 4 | 4 | 0.25 | 2 | 8 | on | 1.325 | 99.33 | no |
| 4 | 4 | 0.25 | 2 | None | off | 1.325 | 99.24 | no |
| 4 | 4 | 0.25 | 2 | None | on | 1.325 | 99.33 | no |
| 4 | 4 | 0.25 | 3 | 3 | off | 0.663 | 96.94 | yes |
| 4 | 4 | 0.25 | 3 | 3 | on | 0.663 | 97.03 | yes |
| 4 | 4 | 0.25 | 3 | 4 | off | 0.663 | 96.94 | yes |
| 4 | 4 | 0.25 | 3 | 4 | on | 0.663 | 97.03 | yes |
| 4 | 4 | 0.25 | 3 | 6 | off | 1.325 | 96.94 | no |
| 4 | 4 | 0.25 | 3 | 6 | on | 1.325 | 97.03 | no |
| 4 | 4 | 0.25 | 3 | 8 | off | 1.325 | 96.97 | no |
| 4 | 4 | 0.25 | 3 | 8 | on | 1.325 | 97.06 | no |
| 4 | 4 | 0.25 | 3 | None | off | 1.325 | 96.97 | no |
| 4 | 4 | 0.25 | 3 | None | on | 1.325 | 97.06 | no |

## dev

| intended | runs | FP/1000 | sentences with FP % | recall % | caught typing % | only at space % | median typing k | p90 typing k | avg ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| th | 3584 | 0.000 | 0.00 | 99.75 | 91.55 | 8.20 | 2 | 3 | 0.0091 |
| en | 2936 | 0.681 | 0.12 | 98.88 | 95.71 | 3.17 | 2.0 | 2 | 0.0041 |
| neutral | 104 | 0.000 | 0.00 | n/a | n/a | n/a | None | None | 0.0037 |

### th worst false positives

- none

### th misses

- run='ณ'; garbled='I'; prev=th; score=-1.470
- run='น'; garbled='o'; prev=th; score=2.573
- run='น'; garbled='o'; prev=None; score=2.573
- run='หนา'; garbled='sok'; prev=en; score=1.100
- run='ก'; garbled='d'; prev=None; score=0.145
- run='ฮ'; garbled='V'; prev=th; score=0.432
- run='ก'; garbled='d'; prev=None; score=0.145
- run='ฮ'; garbled='V'; prev=th; score=0.432
- run='กราฟ'; garbled='dika'; prev=None; score=0.704

### en worst false positives

- run='iur'; at=key 2; prev=en; score=12.015
- run='lu'; at=space; prev=None; score=2.112

### en misses

- run='a'; garbled='ฟ'; prev=en; score=7.594
- run='d'; garbled='ก'; prev=th; score=-0.145
- run='x'; garbled='ป'; prev=None; score=3.423
- run='A'; garbled='ฤ'; prev=None; score=19.374
- run='D'; garbled='ฏ'; prev=None; score=36.689
- run='E'; garbled='ฎ'; prev=None; score=2.499
- run='pe'; garbled='ยำ'; prev=en; score=1.132
- run='Ewa'; garbled='ฎไฟ'; prev=None; score=0.733
- run='Ewa'; garbled='ฎไฟ'; prev=None; score=0.733
- run='K'; garbled='ษ'; prev=None; score=31.998
- run='L'; garbled='ศ'; prev=None; score=1.505
- run='a'; garbled='ฟ'; prev=en; score=7.594
- run='M'; garbled='?'; prev=None; score=0.000
- run='MMM'; garbled='???'; prev=None; score=0.000
- run='R'; garbled='ฑ'; prev=None; score=3.572
- run='iur'; garbled='รีพ'; prev=en; score=-4.659
- run='fuga.'; garbled='ดีเฟใ'; prev=en; score=1.637
- run='S'; garbled='ฆ'; prev=None; score=5.809
- run='T'; garbled='ธ'; prev=None; score=0.157
- run='X'; garbled=')'; prev=None; score=0.000
- run='X'; garbled=')'; prev=None; score=0.000
- run='X'; garbled=')'; prev=None; score=0.000
- run='d'; garbled='ก'; prev=None; score=-0.145
- run='dd'; garbled='กก'; prev=None; score=-1.023
- run='MMM'; garbled='???'; prev=en; score=0.000

### neutral worst false positives

- none

### neutral misses

- none

## test

| intended | runs | FP/1000 | sentences with FP % | recall % | caught typing % | only at space % | median typing k | p90 typing k | avg ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| th | 2811 | 0.356 | 0.06 | 99.89 | 87.69 | 12.20 | 2 | 3 | 0.0124 |
| en | 5162 | 11.817 | 2.45 | 96.73 | 94.56 | 2.17 | 2 | 2 | 0.0046 |
| neutral | 1311 | 0.000 | 0.00 | n/a | n/a | n/a | None | None | 0.0033 |

### th worst false positives

- run='กำนดไฟล์'; at=key 3; prev=en; score=4.700

### th misses

- run='น'; garbled='o'; prev=None; score=2.573
- run='ปณ'; garbled='xI'; prev=th; score=1.051
- run='เหรินหมินปี้'; garbled='gsibos,boxuh'; prev=th; score=1.746

### en worst false positives

- run='i5'; at=key 2; prev=en; score=15.174
- run='i5'; at=key 2; prev=en; score=15.174
- run='i9'; at=key 2; prev=en; score=9.788
- run='i9'; at=key 2; prev=en; score=9.788
- run='i9'; at=key 2; prev=en; score=9.788
- run='i7'; at=key 2; prev=en; score=8.004
- run='i7'; at=key 2; prev=en; score=8.004
- run='-v'; at=key 2; prev=None; score=7.722
- run='-v'; at=key 2; prev=None; score=7.722
- run='-v'; at=key 2; prev=None; score=7.722
- run='-v'; at=key 2; prev=None; score=7.722
- run='-vv'; at=key 2; prev=None; score=7.722
- run='E5'; at=key 2; prev=en; score=7.281
- run='E5'; at=key 2; prev=en; score=7.281
- run='E5'; at=key 2; prev=en; score=7.281
- run='E5'; at=key 2; prev=en; score=7.281
- run='-k'; at=key 2; prev=None; score=5.613
- run='-j'; at=key 2; prev=None; score=5.320
- run='-u'; at=key 2; prev=None; score=5.178
- run='-u'; at=key 2; prev=None; score=5.178
- run='-u'; at=key 2; prev=None; score=5.178
- run='-u'; at=key 2; prev=None; score=5.178
- run='-u'; at=key 2; prev=None; score=5.178
- run='-u'; at=key 2; prev=None; score=5.178
- run='-u'; at=key 2; prev=None; score=5.178

### en misses

- run='-i'; garbled='ขร'; prev=None; score=1.434
- run='-i'; garbled='ขร'; prev=None; score=1.434
- run='-i'; garbled='ขร'; prev=None; score=1.434
- run='-i'; garbled='ขร'; prev=None; score=1.434
- run='-i'; garbled='ขร'; prev=None; score=1.434
- run='-j'; garbled='ข่'; prev=None; score=-3.735
- run='-k'; garbled='ขา'; prev=None; score=-4.222
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433

### neutral worst false positives

- none

### neutral misses

- none

## chat

| intended | runs | FP/1000 | sentences with FP % | recall % | caught typing % | only at space % | median typing k | p90 typing k | avg ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| th | 360 | 19.444 | 4.55 | 97.78 | 88.89 | 8.89 | 2.0 | 3 | 0.0129 |
| en | 348 | 0.000 | 0.00 | 97.13 | 87.93 | 9.20 | 2.0 | 3 | 0.0061 |
| neutral | 0 | n/a | n/a | n/a | n/a | n/a | None | None | 0.0000 |

### th worst false positives

- run='นะครับ'; at=key 2; prev=en; score=8.911
- run='นะ'; at=key 2; prev=en; score=8.911
- run='นะคะ'; at=key 2; prev=en; score=8.911
- run='นะ'; at=key 2; prev=th; score=8.911
- run='นะ'; at=key 2; prev=en; score=8.911
- run='นะคะ'; at=key 2; prev=en; score=8.911
- run='งั้นเอาตามนี้เลยค่ะ'; at=key 2; prev=None; score=5.675

### th misses

- run='ทำ'; garbled='me'; prev=None; score=-0.305
- run='นะ'; garbled='ot'; prev=en; score=-3.511
- run='นะคะ'; garbled='ot8t'; prev=en; score=1.840
- run='นะ'; garbled='ot'; prev=th; score=-3.511
- run='นะ'; garbled='ot'; prev=en; score=-3.511
- run='นะคะ'; garbled='ot8t'; prev=en; score=1.840
- run='เย้'; garbled='gph'; prev=th; score=-1.603
- run='แป๊บนึงนะ'; garbled="cxU[o7'ot"; prev=en; score=1.622

### en worst false positives

- none

### en misses

- run='me'; garbled='ทำ'; prev=en; score=0.305
- run='I'; garbled='ณ'; prev=None; score=1.470
- run='I'; garbled='ณ'; prev=None; score=1.470
- run='me'; garbled='ทำ'; prev=en; score=0.305
- run='a'; garbled='ฟ'; prev=en; score=7.594
- run='I'; garbled='ณ'; prev=None; score=1.470
- run='I'; garbled='ณ'; prev=None; score=1.470
- run='I'; garbled='ณ'; prev=None; score=1.470
- run='a'; garbled='ฟ'; prev=en; score=7.594
- run='a'; garbled='ฟ'; prev=en; score=7.594

### neutral worst false positives

- none

### neutral misses

- none

## Test and chat comparison

| set | intended | version | FP/1000 | recall % | caught typing % | only at space % |
|---|---|---|---:|---:|---:|---:|
| test | th | v1 baseline | 209.18 | 98.01 | n/a | n/a |
| test | en | v1 baseline | 3.10 | 96.55 | n/a | n/a |
| test | th | v2 max_check_len=null | 0.356 | 99.89 | 98.72 | 1.17 |
| test | en | v2 max_check_len=null | 11.817 | 96.73 | 95.99 | 0.74 |
| test | neutral | v2 max_check_len=null | 0.000 | n/a | n/a | n/a |
| test | th | v2 chosen max_check_len | 0.356 | 99.89 | 87.69 | 12.20 |
| test | en | v2 chosen max_check_len | 11.817 | 96.73 | 94.56 | 2.17 |
| test | neutral | v2 chosen max_check_len | 0.000 | n/a | n/a | n/a |
| chat | th | v1 baseline | 186.11 | 95.00 | n/a | n/a |
| chat | en | v1 baseline | 0.00 | 96.55 | n/a | n/a |
| chat | th | v2 max_check_len=null | 19.444 | 97.78 | 96.67 | 1.11 |
| chat | en | v2 max_check_len=null | 0.000 | 97.13 | 93.68 | 3.45 |
| chat | th | v2 chosen max_check_len | 19.444 | 97.78 | 88.89 | 8.89 |
| chat | en | v2 chosen max_check_len | 0.000 | 97.13 | 87.93 | 9.20 |

## Known weaknesses

The dev FP cap did not generalize fully: test English is 11.817 FP/1,000 versus the v1 baseline of 3.10, and chat Thai is 19.444/1,000. Limiting checks to three keys reduces typing checks but does not change run-level FP, since the complete check still runs at space. The n-gram models come from limited local-language corpora, so short names, abbreviations, and uncommon Thai words remain ambiguous. Neutral has no chat candidates, so its chat FP rate is n/a. Run boundaries are heuristic, and chat is a small illustrative sample rather than a representative user study.

## v3: owner's real chats (aggregate numbers only, 2026-11-08)

Data: 3 LINE exports (space-separated format), only the owner's own messages, 16,066 after dedupe, split 70/30 by
hash; a 20% dev split was carved out of the 70% and personal models were retrained without it for tuning. The
30% test split (`me`) was used once, after the config was fixed. No message text is stored in this repo.

Changes: parser supports `HH:MM Sender Message` with spaces in names; bare placeholders (`รูป`, `สติกเกอร์`, …),
HTML tags and zero-width characters are stripped; `orthography_auto=false` (invalid Thai adds evidence instead of
converting immediately); `min_len=4` while typing, `min_len_final=3` at the space bar; `max_check_len=null`.

| set | lang | v2 config, generic model FP/1000 · recall | v3 config, generic model | v3 config, personal model |
|---|---|---|---|---|
| me | th | 28.42 · 93.89 | 7.11 · 91.42 | **1.72 · 93.70** |
| me | en | 7.79 · 96.79 | 5.19 · 85.77 | **5.19 · 85.88** |
| chat | th | 19.44 · 97.78 | 0.00 · 89.44 | 0.00 · 92.78 |
| chat | en | 0.00 · 97.13 | 0.00 · 89.66 | 0.00 · 89.66 |
| test | th | 0.36 · 99.89 | 0.00 · 97.76 | 0.00 · 97.37 |
| test | en | 11.82 · 96.73 | 0.19 · 90.02 | 0.19 · 90.00 |

Personal model on `me`: Thai caught while typing 86.4% (median 4th key, p90 5th), at space 7.3%; English while
typing 69.8%, at space 16.1%. All 5 English "false positives" on `me` decode to meaningful Thai, i.e. real
wrong-layout text the owner left uncorrected, so the true English FP is ~0. The 7 Thai FPs are 5 words that already
contained typos plus a Thai-digit laugh and an interjection. Cost of v3: English recall drops because 1–2 letter
English words are never converted (22 of the 25 listed misses), and detection waits until the 4th key.
