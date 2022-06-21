# Approaching Message Optimal Byzantine Reliable Broadcast using Routing

This code implements simultations from the paper [found here](https://repository.tudelft.nl/islandora/object/uuid:dbb3fb22-a6cd-4994-bf1f-7dab42fcc369).

## Requirements

To run the simulations, you will need to have installed the [Rust programming language](https://www.rust-lang.org/tools/install).
To generate the graphs from the output, you are required to have [Python](https://www.python.org/downloads/) installed.
In addition to that, you need the `parse`, `matplotlib`, and `networkx` Python dependencies.

## Run simulations

This repository provides multiple simulations, given in the `dissyssym` folder.
All of them are normally run in release mode, as the algorithms are computationally very heavy.

### Topology Generation

This binary will generate random `k`-connected graphs.
It will use a node count between `2` and `100`, and generate every possible connectivity for each `n`.
For each possible combination of the parameters, five random topologies are generated.
These topologies are then stored in the `./topologies` folder.
To run it, you can use the following command:

```bash
cargo run --bin gentopology --release
```

### Simulate

Using the topologies generate in the Topology Generation section, this will simulate both algorithms on the topologies.
For this, every valid value for `f` will be evaluated.
Results will be written to the console and `results.data`.

```bash
cargo run --bin simulate --release
```

### Path Timing

This compares the time the two path building algorithms take to generate the paths.
Results will be written to the console and `pathtime.data`.
It can be run by the following command:

```bash
cargo run --bin pathtime --release
```

### Failure

This will simulate the algorithms, and count when and with which parameters simulations failed.
Results will be written to the console and `failures.data`.
It can be run by the following command:

```bash
cargo run --bin failure --release
```

## Build Visulizations

To build the graphs used in the paper, some Python scripts have been made.
All graphs will be written to `{RESULTS_FILE}.png`.

### Simulate

To build the simulation graphs, you can run the following command.
The parameter is the path to the results file.

```bash
python ./scripts/graph_messages.py ./results.data
```

### Path Timing

To build the simulation graphs, you can run the following command.
The parameter is the path to the results file.

```bash
python ./scripts/graph_pathtime.py ./pathtime.data
```

### Failure

To build the failure graphs, you can run the following command.
The parameter is the path to the results file.

```bash
python ./scripts/graph_failure.py ./failures.data
```
