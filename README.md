# skuggbox-rs

## Todo

- [ ] Define project structure
- [ ] Load / save project
- [ ] Export `glsl` using the minifier  
- [ ] Uniforms
- [ ] Read uniforms from fragment shader
- [ ] Implement camera
- [ ] Define object structure based on SDF primitives
- [ ] Pragma import
- [ ] Individually scale, translate, rotate individual primitives
- [ ] Default renderer
- [ ] GI renderer

### SDF primitive structure

```text
root object
  => transform(sdBall(a,b,c)
    => scale(rotate(translate(sdBall(a,b,c), vec3(1.0))
  => translate(sdBall(a,b,c), vec3(1.0))
```


### Run

```bash
set CARGO_INCREMENTAL=1 && set RUSTFLAGS=-C lto=off -C opt-level=z -C inline-threshold=275 && cargo watch -x "run --release"  --ignore './shaders/*'
```

```shell
cargo test
```


### Scratch pad

Build a tree representation of SDF primitives