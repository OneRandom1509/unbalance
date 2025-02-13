# web-server

A multithreaded web server

## How to Use:

1. Run the command:

```
cargo run <number_of_workers>
```

2. Head over to `localhost:7878` on your browser. There are currently two routes available (`/` and `/sleep` the latter being just a 5 second timeout). Any other route will give a 404 page.
