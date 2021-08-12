# Hybrid Metaheuristic solver for the Capacitated Vehicle Routing Problem (CVRP)

The code is an implementation of the Hybrid Genetic Search with Ruin-and-Recreate (HGSRR).

## Running the metaheuristic

The simplest way to run the metaheuristic is with the following command:

```
cargo run --release --bin hybridvrp
```

To enable logging, run the following instead:

```
RUST_LOG=info cargo run --release --bin hybridvrp
```

## Configuration

Please see the `config.yml` file, as it is includes all the parameters required to run the metaheuristic.