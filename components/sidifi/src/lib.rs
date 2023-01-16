use uuid::Uuid;

use crate::glsl::{op_union, sd_box_to_code, translate};
use glam::Vec3;

pub mod errors;
pub mod glsl;

#[derive(Debug, Clone)]
pub enum Primitive {
    Box(Vec3),
}

/// Domain operations such as union, chamfer, etc
#[derive(Debug, Clone)]
pub enum Combination {
    Union,
}

#[derive(Debug, Clone)]
pub enum Transform {
    Translate(Vec3),
}

#[derive(Debug, Clone)]
pub enum Op {
    Root,
    Combination(Combination),
    Transform(Transform),
    Primitive(Primitive),
}

type NodeId = Uuid;

/// A node in the SDF tree
/// Either a primitive or an op
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub op: Op,
    /// TODO(mathias): might not need this
    pub parent_id: Option<NodeId>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new_root() -> Self {
        Self {
            id: NodeId::new_v4(),
            op: Op::Root,
            parent_id: None,
            children: vec![],
        }
    }

    pub fn from_primitive(primitive: Primitive) -> Self {
        Self {
            id: NodeId::new_v4(),
            op: Op::Primitive(primitive),
            parent_id: None,
            children: vec![],
        }
    }

    pub fn from_op(op: Combination) -> Self {
        Self {
            id: NodeId::new_v4(),
            op: Op::Combination(op),
            parent_id: None,
            children: vec![],
        }
    }

    pub fn from_transform(transform: Transform) -> Self {
        Self {
            id: Default::default(),
            op: Op::Transform(transform),
            parent_id: None,
            children: vec![],
        }
    }

    // TODO(mathias): Pass Node by reference instead
    pub fn add_child(&mut self, mut node: Node) {
        node.parent_id = Some(self.id);
        self.children.push(node);
    }

    pub fn to_code(&self) -> String {
        match &self.op {
            Op::Root => {
                // TODO(mathias): Traverse all children nodes and replace all placeholders
                self.children
                    .iter()
                    .map(|child| child.to_code())
                    .collect::<Vec<String>>()
                    .join("")
            }
            Op::Combination(op) => match op {
                Combination::Union => {
                    let node_a = self.children.get(0).expect("union needs two children");
                    let node_b = self.children.get(1).expect("union needs two children");
                    op_union(node_a, node_b)
                }
            },
            // Ideally a transform node should only have one child.
            // This code does not account for more than one child at this point in time.
            Op::Transform(transform) => match transform {
                // TODO(mathias): Return error if the transform doesn't have a child
                Transform::Translate(vec4) => self
                    .children
                    .iter()
                    .map(|child| {
                        child
                            .to_code()
                            .replace("_POSITION_", translate(vec4).as_str())
                    })
                    .collect::<Vec<String>>()
                    .join(""),
            },
            Op::Primitive(primitive) => match primitive {
                Primitive::Box(size) => sd_box_to_code(size),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_child() {
        let vec3 = Vec3::new(1.1, 1.2, 1.3);
        let mut root = Node::new_root();
        let node = Node::from_primitive(Primitive::Box(vec3));
        root.add_child(node.clone());

        assert_eq!(root.children.len(), 1);
    }

    // primitives
    #[test]
    fn primitive_box_to_code() {
        let vec3 = Vec3::new(1.1, 1.2, 1.3);
        let mut root = Node::new_root();
        let node = Node::from_primitive(Primitive::Box(vec3));
        root.add_child(node.clone());

        assert_eq!(node.to_code(), "sdBox(_POSITION_, vec3(1.1, 1.2, 1.3))");
    }

    /// Translating (or any transforms of the position) need to inline into where `p` is.
    /// These test builds a straightforward tree with unions and translations.
    ///
    /// root
    ///   union
    ///     translate
    ///       box
    ///
    #[test]
    fn transform_translate_to_code_basic() {
        let mut root = Node::new_root();

        let translate = Vec3::new(0.5, 0.5, 0.5);
        let mut transform_a = Node::from_transform(Transform::Translate(translate));

        let vec3 = Vec3::new(1.1, 1.2, 1.3);
        let box_a = Node::from_primitive(Primitive::Box(vec3));
        transform_a.add_child(box_a);

        assert_eq!(
            "sdBox(translate(_POSITION_, vec3(0.5, 0.5, 0.5)), vec3(1.1, 1.2, 1.3))",
            transform_a.to_code()
        );
    }

    ///
    ///  root
    ///    union_a
    ///      translate_a
    ///        union_b
    ///          translate_b
    ///            box_c
    ///          box_b
    ///      box_a
    ///
    #[test]
    fn transform_translate_to_code_children() {
        let pos_vec3 = Vec3::new(0.5, 0.5, 0.5);
        let size_vec3 = Vec3::new(1.1, 1.2, 1.3);

        let mut root = Node::new_root();
        let translate_a = Node::from_transform(Transform::Translate(pos_vec3));
        let mut translate_b = Node::from_transform(Transform::Translate(pos_vec3));
        let mut union_a = Node::from_op(Combination::Union);
        let mut union_b = Node::from_op(Combination::Union);
        let box_a = Node::from_primitive(Primitive::Box(size_vec3));
        let box_b = Node::from_primitive(Primitive::Box(size_vec3));
        let box_c = Node::from_primitive(Primitive::Box(size_vec3));

        translate_b.add_child(box_c);
        union_b.add_child(translate_b);
        union_b.add_child(box_b);

        union_a.add_child(box_a.clone());
        union_a.add_child(union_b.clone());

        root.add_child(union_a.clone());

        assert_eq!(
          "min(sdBox(_POSITION_, vec3(1.1, 1.2, 1.3)), min(sdBox(translate(_POSITION_, vec3(0.5, 0.5, 0.5)), vec3(1.1, 1.2, 1.3)), sdBox(_POSITION_, vec3(1.1, 1.2, 1.3))))",
            root.to_code()
        );
    }

    #[test]
    fn combination_union_to_code() {
        let mut op = Node::from_op(Combination::Union);

        let vec3 = Vec3::new(1.1, 1.2, 1.3);
        let node_a = Node::from_primitive(Primitive::Box(vec3));
        let node_b = Node::from_primitive(Primitive::Box(vec3));

        op.add_child(node_a.clone());
        op.add_child(node_b.clone());

        // assert_eq!(op.to_code(), "translate(_POSITION_, vec3(1.1, 1.2, 1.3)");
        // assert_eq!(
        //     "min(sdBox(translate(_POSITION_, vec3(1.1, 1.2, 1.3)), vec3(1.1, 1.2, 1.3)))",
        //     op.to_code()
        // )
    }

    #[test]
    fn node_tree_parents() {
        let root = Node::new_root();
        let mut op = Node::from_op(Combination::Union);

        let vec3 = Vec3::new(1.1, 1.2, 1.3);
        let node_a = Node::from_primitive(Primitive::Box(vec3));
        let node_b = Node::from_primitive(Primitive::Box(vec3));

        op.add_child(node_a.clone());
        op.add_child(node_b.clone());

        for node in op.children.iter() {
            assert_eq!(node.parent_id.unwrap(), op.id);
        }
    }
}
