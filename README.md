# BenchmarkJan2026
Python, Go, and Rust code for benchmarking execution time

## Python
Enter python subdirectory and run main.py for benchmarking results
```
python main.py
```
Adjust N and W in main.py to adjust cases and # of parallel processes.

## Go
Enter go subdirectory and run main.go for benchmarking results
```
go run .
```
Adjust numTasks and numWorkers in main.go to adjust cases and # of parallel processes.

## Rust
Enter rust subdirectory and run using cargo

For running in debug mode (faster compilation, slower execution):
```
cargo run .
```
For running release (slower compilation, faster execution)
```
cargo run --release .
```