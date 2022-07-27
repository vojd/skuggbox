# skuggbox

```bash

cargo run --release -- ./path/to/shader.glsl

## with cargo watch

cargo install cargo-watch
cargo watch -x "run --release"

```

### Run

`cargo run --release shader.glsl`

Optional params:

```text
# load fragment shader
-f /path/to/shader.glsl

# create new fragment shader
-n /path/to/shader.glsl

-a     window is always on top
```

For all params:

```bash
cargo run --release -- --help
```

### Run tests

`cargo test`

### Misc
See file `.ignore` for directories and files ignored by `cargo watch`
