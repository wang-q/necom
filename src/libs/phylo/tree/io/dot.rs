//! Graphviz DOT format writer.

use super::super::Tree;
use std::fmt::Write as _;

/// Escape a string for safe use inside a DOT double-quoted label.
fn escape_dot_label(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// Serialize the tree to a Graphviz DOT string.
pub fn to_dot(tree: &Tree) -> String {
    let mut s = String::from("digraph Tree {\n");
    s.push_str("    node [shape=box];\n"); // Optional styling

    if let Some(root) = tree.get_root() {
        let nodes = tree.preorder(root);
        for &node_id in &nodes {
            let Some(node) = tree.get_node(node_id) else {
                continue;
            };

            // 1. Define Node
            // Use NodeID as the DOT identifier
            let label = node.name.as_deref().unwrap_or("");
            if label.is_empty() {
                let _ = writeln!(s, "    {} [label=\"{}\"];", node_id, node_id);
            } else {
                let _ = writeln!(
                    s,
                    "    {} [label=\"{}\"];",
                    node_id,
                    escape_dot_label(label)
                );
            }

            // 2. Define Edges to children
            for &child_id in &node.children {
                let Some(child) = tree.get_node(child_id) else {
                    continue;
                };

                let _ = write!(s, "    {} -> {}", node_id, child_id);
                if let Some(len) = child.length {
                    if len.is_finite() && len > 0.0 {
                        let _ = write!(s, " [label=\"{}\"]", len);
                    }
                }
                let _ = writeln!(s, ";");
            }
        }
    }

    s.push_str("}\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_dot() {
        let mut tree = Tree::new();
        let n0 = tree.add_node();
        let n1 = tree.add_node();

        let _ = tree.set_root(n0);
        tree.add_child(n0, n1).unwrap();

        tree.get_node_mut(n0).unwrap().set_name("Root");
        tree.get_node_mut(n1).unwrap().set_name("A");
        tree.get_node_mut(n1).unwrap().length = Some(0.1);

        let dot = to_dot(&tree);
        assert!(dot.contains("digraph Tree {"));
        assert!(dot.contains(&format!("{} [label=\"Root\"];", n0)));
        assert!(dot.contains(&format!("{} [label=\"A\"];", n1)));
        assert!(dot.contains(&format!("{} -> {} [label=\"0.1\"];", n0, n1)));
    }

    #[test]
    fn test_to_dot_negative_length() {
        let mut tree = Tree::new();
        let n0 = tree.add_node();
        let n1 = tree.add_node();

        let _ = tree.set_root(n0);
        tree.add_child(n0, n1).unwrap();

        tree.get_node_mut(n0).unwrap().set_name("Root");
        tree.get_node_mut(n1).unwrap().set_name("A");
        tree.get_node_mut(n1).unwrap().length = Some(-0.5);

        let dot = to_dot(&tree);
        // Negative length should be treated as 0.0 (no label emitted)
        assert!(!dot.contains("label=\"-0.5\""));
        // Edge should exist but without label attribute
        assert!(dot.contains(&format!("{} -> {};", n0, n1)));
    }

    #[test]
    fn test_escape_dot_label() {
        // Backslashes and double quotes must be escaped inside DOT labels.
        assert_eq!(escape_dot_label("A\\B"), "A\\\\B");
        assert_eq!(escape_dot_label("A\"B"), "A\\\"B");
        assert_eq!(escape_dot_label("A\\\"B"), "A\\\\\\\"B");
        assert_eq!(escape_dot_label("plain"), "plain");

        // Real whitespace characters must be escaped so the label stays on one
        // line inside the DOT file.
        assert_eq!(escape_dot_label("A\nB"), "A\\nB");
        assert_eq!(escape_dot_label("A\rB"), "A\\rB");
        assert_eq!(escape_dot_label("A\tB"), "A\\tB");
    }

    #[test]
    fn test_to_dot_escapes_special_chars() {
        let mut tree = Tree::new();
        let n0 = tree.add_node();
        let n1 = tree.add_node();

        let _ = tree.set_root(n0);
        tree.add_child(n0, n1).unwrap();

        tree.get_node_mut(n1).unwrap().set_name("A\"B\\C");

        let dot = to_dot(&tree);
        assert!(dot.contains("label=\"A\\\"B\\\\C\""));
    }

    #[test]
    fn test_to_dot_zero_length() {
        let mut tree = Tree::new();
        let n0 = tree.add_node();
        let n1 = tree.add_node();

        let _ = tree.set_root(n0);
        tree.add_child(n0, n1).unwrap();

        tree.get_node_mut(n0).unwrap().set_name("Root");
        tree.get_node_mut(n1).unwrap().set_name("A");
        tree.get_node_mut(n1).unwrap().length = Some(0.0);

        let dot = to_dot(&tree);
        // Zero length should be treated as no label
        assert!(!dot.contains("label=\"0\""));
        assert!(dot.contains(&format!("{} -> {};", n0, n1)));
    }

    #[test]
    fn test_to_dot_missing_length() {
        let mut tree = Tree::new();
        let n0 = tree.add_node();
        let n1 = tree.add_node();

        let _ = tree.set_root(n0);
        tree.add_child(n0, n1).unwrap();

        tree.get_node_mut(n0).unwrap().set_name("Root");
        tree.get_node_mut(n1).unwrap().set_name("A");
        // length is None by default

        let dot = to_dot(&tree);
        // Missing length should produce no edge label attribute.
        assert!(!dot.contains(&format!("{} -> {} [label=", n0, n1)));
        assert!(dot.contains(&format!("{} -> {};", n0, n1)));
    }
}
