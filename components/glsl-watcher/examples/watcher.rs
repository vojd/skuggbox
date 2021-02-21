use glsl_watcher::load;

pub fn main() {
    println!("glsl loader");

    load(
        "./examples".to_string(),
        "base.vert".to_string(),
        "base.frag".to_string(),
    );
}
