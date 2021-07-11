# skuggbox-rs

```bash

cargo run --release

## with cargo watch

cargo install cargo-watch
cargo watch -x "run --release"
 
```

See file `.ignore` for directories and files ignored by `cargo watch`

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



### Scratch pad

Build a tree representation of SDF primitives
