//! Newick format reader and writer.

use super::super::Tree;
use crate::libs::phylo::node::NodeId;
use crate::libs::phylo::parser::is_newick_reserved;
use std::fmt::Write as _;
use std::io::Read;

/// Read a Newick tree from a file.
///
/// # Arguments
/// * `infile` - Path to the input file (or "stdin" for stdin).
///
/// # Example
/// ```
/// // usage in CLI:
/// // let trees = necom::libs::phylo::tree::io::from_file("path/to/tree.nwk")?;
/// ```
pub fn from_file(infile: &str) -> anyhow::Result<Vec<Tree>> {
    let mut reader = crate::reader(infile)?;
    let mut newick = String::new();
    reader
        .read_to_string(&mut newick)
        .map_err(|e| anyhow::anyhow!("Read error: {}", e))?;
    Tree::from_newick_multi(newick.as_str())
        .map_err(|e| anyhow::anyhow!("failed to parse '{}': {}", infile, e))
}

/// Serialize tree to Newick string.
pub fn to_newick(tree: &Tree) -> String {
    to_newick_with_format(tree, "")
}

/// Serialize tree to Newick string with custom formatting options.
/// Currently supports indentation (empty for single line).
pub fn to_newick_with_format(tree: &Tree, indent: &str) -> String {
    if let Some(root) = tree.get_root() {
        let mut s = String::new();
        to_newick_recursive(tree, root, indent, 0, &mut s);
        s.push(';');
        s
    } else {
        String::new()
    }
}

/// Serialize a specific subtree to a Newick string.
pub fn to_newick_subtree(tree: &Tree, root: NodeId, indent: &str) -> String {
    let mut s = String::new();
    to_newick_recursive(tree, root, indent, 0, &mut s);
    s.push(';');
    s
}

fn to_newick_recursive(tree: &Tree, node_id: NodeId, indent: &str, depth: usize, s: &mut String) {
    let Some(node) = tree.get_node(node_id) else {
        return;
    };
    let is_pretty = !indent.is_empty();

    // Calculate current indentation string
    let my_indent = if is_pretty {
        indent.repeat(depth)
    } else {
        String::new()
    };

    if node.children.is_empty() {
        // Leaf: Indent + NodeInfo
        let _ = write!(s, "{}{}", my_indent, node_info(tree, node_id));
    } else {
        // Internal node
        if is_pretty {
            let _ = writeln!(s, "{}(", my_indent);
        } else {
            s.push('(');
        }
        for (i, &child) in node.children.iter().enumerate() {
            to_newick_recursive(tree, child, indent, depth + 1, s);
            if i + 1 < node.children.len() {
                if is_pretty {
                    s.push_str(",\n");
                } else {
                    s.push(',');
                }
            }
        }
        if is_pretty {
            let _ = write!(s, "\n{}){}", my_indent, node_info(tree, node_id));
        } else {
            let _ = write!(s, "){}", node_info(tree, node_id));
        }
    }
}

/// Format node info: Label + Length + Comment.
fn node_info(tree: &Tree, node_id: NodeId) -> String {
    let Some(node) = tree.get_node(node_id) else {
        return String::new();
    };

    let mut node_info = String::new();

    if let Some(name) = &node.name {
        node_info.push_str(&quote_label(name));
    }

    let len = node.finite_length();
    if len > 0.0 {
        let _ = write!(node_info, ":{}", len);
    }
    // len == 0.0 (including NaN/inf/negative inputs) is omitted to keep
    // cladograms clean and match the documented output semantics.

    if let Some(props) = &node.properties {
        if !props.is_empty() {
            node_info.push_str("[&&NHX");
            for (k, v) in props {
                if v.is_empty() {
                    let _ = write!(node_info, ":{}", k);
                } else {
                    let _ = write!(node_info, ":{}={}", k, escape_nhx_value(v));
                }
            }
            node_info.push(']');
        }
    }

    node_info
}

fn quote_label(label: &str) -> String {
    let needs_quote = label
        .chars()
        .any(|c| is_newick_reserved(c) || c == '\'' || c == '"' || c.is_whitespace());
    if needs_quote {
        format!("'{}'", label.replace('\'', "''"))
    } else {
        label.to_string()
    }
}

/// Escape characters that are special inside an NHX annotation value.
///
/// Backslashes and closing square brackets must be escaped so that the
/// generated Newick string can be re-parsed unambiguously.
pub fn escape_nhx_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ']' => out.push_str("\\]"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_newick() {
        let mut tree = Tree::new();
        //    Root
        //   /    \
        //  A:0.1  B:0.2
        let n0 = tree.add_node();
        let n1 = tree.add_node();
        let n2 = tree.add_node();

        let _ = tree.set_root(n0);
        tree.add_child(n0, n1).unwrap();
        tree.add_child(n0, n2).unwrap();

        tree.get_node_mut(n0).unwrap().set_name("Root");
        tree.get_node_mut(n1).unwrap().set_name("A");
        tree.get_node_mut(n1).unwrap().length = Some(0.1);
        tree.get_node_mut(n2).unwrap().set_name("B");
        tree.get_node_mut(n2).unwrap().length = Some(0.2);

        // Compact output
        assert_eq!(to_newick(&tree), "(A:0.1,B:0.2)Root;");

        // Pretty output
        let expected_pretty = "(\n  A:0.1,\n  B:0.2\n)Root;";
        assert_eq!(to_newick_with_format(&tree, "  "), expected_pretty);
    }

    #[test]
    fn test_to_newick_complex() {
        let mut tree = Tree::new();
        //      Root
        //     /    \
        //    I1     C
        //   /  \
        //  A    B
        let root = tree.add_node();
        let i1 = tree.add_node();
        let c = tree.add_node();
        let a = tree.add_node();
        let b = tree.add_node();

        let _ = tree.set_root(root);
        tree.get_node_mut(root).unwrap().set_name("Root");

        tree.add_child(root, i1).unwrap();
        tree.add_child(root, c).unwrap();
        tree.get_node_mut(i1).unwrap().set_name("I1");
        tree.get_node_mut(c).unwrap().set_name("C");

        tree.add_child(i1, a).unwrap();
        tree.add_child(i1, b).unwrap();
        tree.get_node_mut(a).unwrap().set_name("A");
        tree.get_node_mut(b).unwrap().set_name("B");

        // Pretty output with tab indentation
        let expected = "(\n\t(\n\t\tA,\n\t\tB\n\t)I1,\n\tC\n)Root;";
        assert_eq!(to_newick_with_format(&tree, "\t"), expected);
    }

    #[test]
    fn test_to_newick_special_chars() {
        let mut tree = Tree::new();
        let n0 = tree.add_node();
        let _ = tree.set_root(n0);
        tree.get_node_mut(n0).unwrap().set_name("Homo sapiens");

        assert_eq!(to_newick(&tree), "'Homo sapiens';");

        tree.get_node_mut(n0).unwrap().set_name("func(x)");
        assert_eq!(to_newick(&tree), "'func(x)';");
    }

    #[test]
    fn test_to_newick_properties() {
        let mut tree = Tree::new();
        let n0 = tree.add_node();
        let _ = tree.set_root(n0);
        tree.get_node_mut(n0).unwrap().set_name("A");
        tree.get_node_mut(n0).unwrap().add_property("color", "red");

        let output = to_newick(&tree);
        // Since BTreeMap order is deterministic (alphabetical keys), but we only have one key here.
        assert!(output.contains("A[&&NHX:color=red];"));
    }

    #[test]
    fn test_to_newick_property_value_escaping_round_trip() {
        let mut tree = Tree::new();
        let n0 = tree.add_node();
        let _ = tree.set_root(n0);
        tree.get_node_mut(n0).unwrap().set_name("A");
        tree.get_node_mut(n0)
            .unwrap()
            .add_property("comment", "a]b\\c");

        let output = to_newick(&tree);
        assert!(output.contains("A[&&NHX:comment=a\\]b\\\\c];"));

        let parsed = Tree::from_newick(&output).unwrap();
        let root = parsed.get_node(parsed.get_root().unwrap()).unwrap();
        assert_eq!(
            root.get_property("comment").map(|s| s.as_str()),
            Some("a]b\\c")
        );
    }

    #[test]
    fn test_to_newick_non_finite_lengths() {
        let mut tree = Tree::new();
        let root = tree.add_node();
        let a = tree.add_node();
        let _ = tree.set_root(root);
        tree.add_child(root, a).unwrap();

        tree.get_node_mut(root).unwrap().set_name("Root");
        tree.get_node_mut(a).unwrap().set_name("A");

        // NaN, infinity, negative, and zero lengths are all omitted on output.
        tree.get_node_mut(a).unwrap().length = Some(f64::NAN);
        assert_eq!(to_newick(&tree), "(A)Root;");

        tree.get_node_mut(a).unwrap().length = Some(f64::INFINITY);
        assert_eq!(to_newick(&tree), "(A)Root;");

        tree.get_node_mut(a).unwrap().length = Some(f64::NEG_INFINITY);
        assert_eq!(to_newick(&tree), "(A)Root;");

        tree.get_node_mut(a).unwrap().length = Some(-1.0);
        assert_eq!(to_newick(&tree), "(A)Root;");

        tree.get_node_mut(a).unwrap().length = Some(0.0);
        assert_eq!(to_newick(&tree), "(A)Root;");

        tree.get_node_mut(a).unwrap().length = Some(0.5);
        assert_eq!(to_newick(&tree), "(A:0.5)Root;");
    }

    #[test]
    fn test_quote_label_escaping() {
        // Whitespace and reserved characters force quoting.
        assert_eq!(quote_label("Homo sapiens"), "'Homo sapiens'");
        assert_eq!(quote_label("A(B)"), "'A(B)'");

        // Single quotes are escaped by doubling.
        assert_eq!(quote_label("A'B"), "'A''B'");

        // Double quotes force quoting too.
        assert_eq!(quote_label("A\"B"), "'A\"B'");

        // Plain labels pass through unchanged.
        assert_eq!(quote_label("AB_123"), "AB_123");
    }

    #[test]
    fn test_to_newick_round_trip_quoted_labels() {
        // Labels that require quoting should round-trip through the parser.
        let labels = [
            "Homo sapiens",
            "A'B",
            "A\"B",
            "A(B)",
            "A,B",
            "A:B",
            "A;B",
            "A[B",
            "A]B",
        ];
        for label in &labels {
            let newick = format!("('{}');", label.replace('\'', "''"));
            let trees = Tree::from_newick_multi(&newick).unwrap();
            assert_eq!(trees.len(), 1, "failed to parse: {}", newick);
            let tree = &trees[0];
            let root = tree.get_root().unwrap();
            let child = tree
                .get_node(tree.get_node(root).unwrap().children[0])
                .unwrap();
            assert_eq!(child.name.as_deref(), Some(*label));
        }
    }
}
