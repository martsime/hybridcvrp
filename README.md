
# HybridCVRP

HybridCVRP is a metaheuristic solver for the Capacitated Vehicle Routing Problem (CVRP).
It contains an implementation of the Hybrid Genetic Search with Ruin-and-Recreate [1].

## Demo
A demo of the solver can be found at [https://martsime.github.io/hybridcvrp](https://martsime.github.io/hybridcvrp).

![HybridCVRP](https://user-images.githubusercontent.com/14152372/131351266-38837f93-e117-4aec-b54b-bc69064057e1.gif)

For the demo webpage, the solver is compiled to WebAssembly and run in the browser.
More information about the demo can be found here: [https://github.com/martsime/hybridcvrp-web](https://github.com/martsime/hybridcvrp-web).
Despite the demo being fully functional, running the solver compiled to WebAssembly in the browser may come with a signficiant performance drawback.
Therefore, for any use other than demo purposes, it is recommended to install the solver and run it locally as described below.


## Running the solver
As the solver is implemented in Rust, you are required to have a [Rust installation](https://www.rust-lang.org/) to build it.
A Rust version of (1.54.0 stable) or newer is required, although older versions could be supported.

The simplest way to run the solver is with the following command:

```
cargo run --release <instance-path>
```

For example:
```
cargo run --release instances/X-n101-k25.vrp
```

## Configuration

The best way to configurate the solver is by changing the parameter values in the `config.yml` file, which is parsed by the solver at startup.

There are also a few optional arguments on the run command. Run `cargo run --release -- --help` to see more information about run command and its arguments. Note that the provided arguments to the run command will take precedence over the parameter values set in `config.yml`.

## Acknowledgments

Huge thanks to Thibaut Vidal for open sourcing an implementation of the [Hybrid Genetic Search specialized for the CVRP](https://github.com/vidalt/HGS-CVRP).
Access to the implementation was very helpfull during the development of the HybridCVRP solver.

## References
[1] Simensen, M., Hasle, G. & Stålhane, M. Combining hybrid genetic search with ruin-and-recreate for solving the capacitated vehicle routing problem. *Journal of Heuristics* (2022). [https://doi.org/10.1007/s10732-022-09500-9](https://doi.org/10.1007/s10732-022-09500-9)
