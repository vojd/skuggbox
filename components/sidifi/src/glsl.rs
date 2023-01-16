use crate::Node;
use glam::Vec3;

/// glsl to code
///
/// All functions in the glsl-code that takes `p` as a parameter must be free of side-effects,
/// otherwise we'll mutate the `p` variable throughout the code in un-wanted ways.
///
/// There are some requirements on the hierarchy:
/// combinatorial operators must be at the top of a tree
/// primitives must be at the bottom of a tree
/// transforms somewhere in between
///
/// _POSITION_ is replaced either by transform nodes or the root node
/// transform nodes:
///   replaces _POSITION_ with transform(p, vec3(...)))
/// root node:
///   replaces _POSITION_ with p

// Primitives
pub(crate) fn sd_box_to_code(dim: &Vec3) -> String {
    format!("sdBox(_POSITION_, vec3({}, {}, {}))", dim.x, dim.y, dim.z)
}

// Combinatorial operators
pub(crate) fn op_union(node_a: &Node, node_b: &Node) -> String {
    format!("min({}, {})", node_a.to_code(), node_b.to_code())
}

// Spatial transforms

///   reverse order from how the nodes are arranged
///   scale(vec3(1.0),
///     rotate(vec3(0.0),
///       translate(vec3(1.0),
///         sdSphere(p, vec3(1.0))
pub(crate) fn translate(vec3: &Vec3) -> String {
    format!(
        "translate(_POSITION_, vec3({}, {}, {}))",
        vec3.x, vec3.y, vec3.z
    )
}
