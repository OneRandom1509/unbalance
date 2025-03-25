# Unbalance

A multi-threaded load-balancer written in Rust.

## How to Use:
1. Type down your workers IPs with `:3424` port in the `.env` file inside `load-balancer` crate

For example:
```
111.111.111.111:3424
222.222.222.222:3424
...
```
2. Run the worker crates in your respective devices using the command `cargo run`.
3. Finally, boot up the load balancer by running the command `cargo run` again. The load balancer crate will automatically identify how many workers will be there and create a threadpool of that size to interact with the workers.
> [!NOTE]  
> If you just want to do a health-check on all your worker nodes, just run the command `cargo run -- -h` or `cargo run -- --health-checker`
