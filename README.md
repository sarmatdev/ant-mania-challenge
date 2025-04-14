# Ant Mania Simulation

- **Ant Spawning:**

  - Ants only spawn in active colonies.
  - It uses a reproducible random seed to make sure the ant placement are consistent across runs for cases when reproductibility is necesary(test, bechmarks, etc...).

- **Ant Movement & Collision:**

  - Each ant moves randomly along available exits each turn.
  - If an ant is in a destroyed (inactive) colony, its marked as dead immediatly.
  - When two or more ants go to the same colony, then that colony is destroyed and all the ants there dies.
  - Ants stop moving after they moved 10,000 times or if all ants are dead.

- **Concurrency Considerations:**
  - The ant movement are run simulation in parallel using Rayon.
  - Random number generator is seeded per thread/chunk.

Code inside is partialy documented, all functions are provided with descriptions.

## How to run:

## Notice, `seed` arg here is optional, if not provided it uses `1970` by default.

The final report does not prints directly to the terminal to prevent terminal bloat, instead, final state with all measurements is stored in the `hive_status.log` file.

```
Seed: 1970
Ants: 100
Map: hiveum_map_small.txt
Simulation time: 398.125µs

Hive status:
Larvonthi east=Kara
Mari west=Ciiaescyg
Varlarbos west=Cosma
```

Simple run:

```sh
cargo run -- --map hiveum_map_small.txt --ants 100
```

Seed provided:

```sh
cargo run -- --map hiveum_map_small.txt --ants 100 --seed 420
```

## Tests and benchmarks:

Run tests:

```sh
cargo test
```

Run benches:

```sh
cargo bench
```

### Benchmark results

For benchmark with `hiveum_map_small` map, `ants=100` and `seed=420`:

```
time:   [298.11 µs 310.49 µs 324.91 µs]
change: [-3.8424% +3.2429% +13.173%] (p = 0.47 > 0.05)
No change in performance detected.
```
