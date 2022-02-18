# Non-clairvoyant scheduling with predictions

Install Rust: 

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Reproduce the experiments:

```bash
cargo run --release -- exp1 -n 10 -l 1000 -a 1.1 -o exp1.csv -p 1 --base-sigma 1.1 --num-sigma 100
cargo run --release -- exp2 -n 10 -l 1000 -a 1.1 -t 10 -s 10.0 -o exp2.csv --rel-sigma
cargo run --release -- exp3 -n 10 -l 1000 -m 5 -o exp3.csv --base-sigma 1.1 --num-sigma 100 -s 100
```

Create plots (requires Python 3 and `seaborn`, install via `pip install seaborn`):

```bash
python3 plot.py exp1.csv --save
python3 plot.py exp2.csv --save
python3 plot.py exp3.csv --save
```
