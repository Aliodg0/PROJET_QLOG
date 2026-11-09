# prog64

A small Rust project that trains a 64-bit linear genetic program (LGP) on a
CartPole environment.

## Overview

- `env` crate: environment trait, CartPole dynamics, deterministic PCG RNG.
- `lgp` crate: 64-bit program representation, mutation, population training,
  serialization, CLI, tests.

A program fits in one `u64`. It contains five 11-bit instructions. Each
instruction reads from four registers or four inputs and writes one register.
Register 0 is the return value.

## Build

```sh
cargo build --workspace
```

## Run

```sh
cargo run -p prog64 -- --population 120 --generations 40 --threads 1 --output best.json
```

## Execution modes

- Single-thread mode (`--threads 1`) evaluates the population directly in the main thread.
- Multi-thread mode (`--threads N`) splits the population into worker threads.
- Both modes use the same ordered population and the same generation seeds, so equivalent configurations produce the same result.

## Training seeds

All programs in a generation use the same common episode seeds. These seeds are generated once per generation from the configured PCG seed. This makes comparisons between individuals fairer and preserves reproducibility.

## CartPole terminal behaviour

After an episode ends, `CartPole::step()` keeps returning a stable terminal result with reward 0 and `done == true`. `reset()` clears the terminal state and starts a new episode. Physical rules and rewards are unchanged.

## Algorithm summary

- Program: 64-bit word, five 11-bit instructions.
- Encoding: 3-bit operation, 2-bit destination, two 3-bit sources.
- Sources: indexes 0-3 are registers; 4-7 are state inputs.
- Registers: initialised to 0; register 0 is the return value.
- Action: `execute(state) > 0.0` pushes right, otherwise left.
- Score: mean cumulative reward over several CartPole episodes.
- CartPole limit: 500 steps per episode.
- Early stop: optional; use `--no-early-stop` for all configured generations.
- Reproducibility: all randomness comes from the configured PCG seed.

## Quality checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Test

```sh
cargo test --workspace
```

## Experimental note

Earlier results are not directly comparable with results produced after changing the training-seed strategy to common seeds.

## License

MIT. See `LICENSE`.

## Fixed training seeds

Training uses one seed list for the whole run:

1. `train()` seeds the population first, preserving the configured initial seed order.
2. It then generates exactly `episodes` common episode seeds once from the same RNG.
3. Every program in every generation is evaluated on that same list.
4. Selection and mutation continue to draw from the same RNG; no new episode seeds are generated per generation.
5. The list is stored in `TrainingReport.training_seeds`.

This keeps within-individual comparisons fair while preserving reproducibility for a given initial seed. Independent evaluation uses a separate seed stream and never reuses training episode seeds.

## Training versus independent evaluation

- Training episodes: the five common seeds generated once per training run.
- Independent evaluation: 100 seeds generated from a separate seed stream (`eval seed seed = 10000`), never used during training.
- A random policy is evaluated on the same 100 independent seeds.
- Reports store training seeds, independent seeds, per-episode scores, aggregate statistics, final program, parameters, and thread count.

## Experimental campaign (final)

Configuration:

- CartPole
- population: 100
- generations: 100
- episodes: 5
- early stopping: disabled
- threads: 1 except one 4-thread comparison run
- seeds: 1, 7, 42 (seed 1 repeated)

Training results:

| Seed | Threads | Training best score | Generations | Final unique programs | Final bit diversity |
|---|---:|---:|---:|---:|---:|
| 1 | 1 | 473.8 | 100 | 64 | 0.2780 |
| 7 | 1 | 500 | 100 | 54 | 0.0886 |
| 42 | 1 | 500 | 100 | 46 | 0.0932 |
| 1 (repeat) | 1 | 473.8 | 100 | 64 | 0.2780 |
| 1 | 4 | 473.8 | 100 | 64 | 0.2780 |

Independent evaluation on 100 seeds:

| Program | Mean | Std | Min | Max |
|---|---:|---:|---:|---:|
| seed 1 | 479.240 | 59.274 | 241 | 500 |
| seed 7 | 500.000 | 0.000 | 500 | 500 |
| seed 42 | 479.240 | 59.274 | 241 | 500 |
| seed 1 repeat | 479.240 | 59.274 | 241 | 500 |
| seed 1 (4 threads) | 479.240 | 59.274 | 241 | 500 |
| random policy | 22.510 | 12.706 | 9 | 74 |

The seed 7 program reaches the 500-step limit on all 100 independent episodes. Other evolved programs are below that level. The evolved programs still substantially exceed the random policy.

## Reproducibility and thread equivalence

- Repeating seed 1 produces the same `best_program`, `best_score`, generation history, unique-program counts, bit diversity, and training seeds.
- The 4-thread run produces the same `best_program`, score, metrics, and training seeds; only the reported thread count differs.
- The per-program 100-episode independent scores also match across the repeated and multithreaded seed 1 runs.

## Genetic diversity versus behavioural diversity

- Bit diversity is a population-level proxy: it measures how evenly bits are distributed across the population for each meaningful bit.
- It is not behavioural diversity. Programs with different bit patterns can implement similar policies, and programs with similar bit patterns can behave differently.
- The reported unique-program count and bit diversity therefore indicate representational diversity, not robustness or behavioural coverage.
- Independent seeds and a random baseline are needed to assess behavioural robustness.

## Limits

- Only CartPole is evaluated, with small state and binary actions.
- Only 5 episodes are used during training, even though evaluation uses 100 episodes.
- A 64-bit program with five instructions has limited expressiveness.
- Common training seeds improve fairness but do not guarantee generalization to unseen seeds.
- The full problem space is not searched extensively; results depend on seed, population size, mutation rate, and generation budget.
- No crossover, no behavioural diversity maintenance, and no static complexity control are used.

## Quality checks

Executed successfully:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```