//! LaTeX Forest format writer.

use super::super::Tree;
use super::util::{compute_depths, compute_heights};
use crate::libs::phylo::node::NodeId;
use std::collections::HashMap;
use std::fmt::Write as _;

/// Escape characters that are special in LaTeX/Forest text.
///
/// This prevents node labels, comments, and property values from breaking
/// Forest syntax or causing LaTeX compilation errors.
fn latex_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str(r"\textbackslash{}"),
            '{' => out.push_str(r"\{"),
            '}' => out.push_str(r"\}"),
            '#' => out.push_str(r"\#"),
            '$' => out.push_str(r"\$"),
            '%' => out.push_str(r"\%"),
            '&' => out.push_str(r"\&"),
            '~' => out.push_str(r"\textasciitilde{}"),
            '^' => out.push_str(r"\textasciicircum{}"),
            _ => out.push(c),
        }
    }
    out
}

/// Convert raw text for LaTeX/Forest display.
///
/// Existing underscores are converted to spaces (matching the documented
/// behavior for `to-forest`/`to-tex`), then LaTeX special characters are
/// escaped.
fn display_text(s: &str) -> String {
    latex_escape(&s.replace('_', " "))
}

/// Serialize tree to LaTeX Forest format.
///
/// # Arguments
/// * `tree` - The tree to serialize.
/// * `height` - Tree height for scaling branch lengths. If 0.0, uses cladogram mode (tier-based).
pub fn to_forest(tree: &Tree, height: f64) -> anyhow::Result<String> {
    if let Some(root) = tree.get_root() {
        let depths = compute_depths(tree);
        let heights = compute_heights(tree);
        let mut s = String::new();
        to_forest_recursive(tree, root, height, &depths, &heights, &mut s)?;
        Ok(s)
    } else {
        Ok(String::new())
    }
}

fn to_forest_recursive(
    tree: &Tree,
    id: NodeId,
    height: f64,
    depths: &HashMap<NodeId, usize>,
    heights: &HashMap<NodeId, usize>,
    s: &mut String,
) -> anyhow::Result<()> {
    let Some(node) = tree.get_node(id) else {
        return Ok(());
    };
    let indent = "  ";

    let children = &node.children;
    let depth = *depths.get(&id).unwrap_or(&0);

    if children.is_empty() {
        let indention = indent.repeat(depth);
        let _ = writeln!(
            s,
            "{}[{}]",
            indention,
            to_forest_node_props(tree, id, height, heights)?
        );
    } else {
        let indention = indent.repeat(depth);
        let _ = writeln!(
            s,
            "{}[{}",
            indention,
            to_forest_node_props(tree, id, height, heights)?
        );
        for &child in children {
            to_forest_recursive(tree, child, height, depths, heights, s)?;
        }
        let _ = writeln!(s, "{}]", indention);
    }
    Ok(())
}

fn to_forest_node_props(
    tree: &Tree,
    id: NodeId,
    height: f64,
    heights: &HashMap<NodeId, usize>,
) -> anyhow::Result<String> {
    let Some(node) = tree.get_node(id) else {
        return Ok(String::new());
    };

    let mut options = String::new();

    let mut name = node.name.clone().map(|x| display_text(&x));
    let mut color: Option<String> = None;
    let mut label: Option<String> = None;

    // internal node's name will be treated as labels and place a dot there
    if !node.is_leaf() && name.is_some() {
        label = name.take();
        // dot with default color
        options += ", dot";
    }

    if let Some(props) = node.properties.as_ref() {
        if let Some(v) = props.get("color") {
            color = Some(v.replace('_', " "));
        }
        if let Some(v) = props.get("label") {
            label = Some(display_text(v));
        }
        for key in ["dot", "bar", "rec", "tri"] {
            if let Some(v) = props.get(key) {
                let _ = write!(options, ", {}={{{}}}", key, display_text(v));
            }
        }
        let mut comment = String::new();
        for key in ["comment", "T", "S", "rank", "member"] {
            if let Some(v) = props.get(key) {
                if !comment.is_empty() {
                    comment += " ";
                }
                comment += &display_text(v);
            }
        }
        if !comment.is_empty() {
            let _ = write!(options, ", comment={{{}}}", comment);
        }
    }

    if let Some(color) = &color {
        if let Some(label) = &label {
            if !label.is_empty() {
                let _ = write!(options, ", label=\\color{{{}}}{{{}}}", color, label);
            }
        }
    } else if let Some(label) = &label {
        if !label.is_empty() {
            let _ = write!(options, ", label={{{}}}", label);
        }
    }

    let mut content = String::new();
    if let Some(color) = &color {
        if let Some(name) = &name {
            let _ = write!(&mut content, "{{\\color{{{}}}{{{}}}}}", color, name);
        } else if node.is_leaf() {
            let _ = write!(&mut content, "{{\\color{{{}}}{{~}}}}", color);
        }
    } else if let Some(name) = &name {
        let _ = write!(&mut content, "{{{}}}", name);
    } else if node.is_leaf() {
        content.push_str("{~}"); // non-breaking space in latex
    }

    if height == 0.0 {
        let tier = *heights.get(&id).unwrap_or(&0);
        let _ = write!(options, ", tier={}", tier);
    } else {
        let edge = node.finite_length();
        let bl = calc_length(edge, height)?;
        let _ = write!(options, ", l={}mm, l sep=0", bl);

        if node.is_leaf() {
            // Add an invisible node to the rightmost to occupy spaces
            options += ", [{~},tier=0,edge={draw=none}]";
        }
    }

    if content.is_empty() {
        // Strip the leading comma separator when there is no node content.
        Ok(if options.starts_with(", ") {
            options.split_off(2)
        } else if options.starts_with(',') {
            options.split_off(1)
        } else {
            options
        })
    } else {
        Ok(content + &options)
    }
}

// relative length
fn calc_length(edge: f64, height: f64) -> anyhow::Result<i32> {
    if height <= 0.0 {
        anyhow::bail!("tree height must be positive for branch-length scaling");
    }
    let scaled = (edge * 100.0 / height).round();
    if !scaled.is_finite() || scaled < i32::MIN as f64 || scaled > i32::MAX as f64 {
        anyhow::bail!(
            "relative branch length {} is out of range for Forest output",
            scaled
        );
    }
    Ok(scaled as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::phylo::tree::Tree;
    use std::collections::BTreeMap;

    #[test]
    fn to_forest_colored_leaf() {
        let mut tree = Tree::new();
        let root = tree.add_node();
        let leaf = tree.add_node();
        let _ = tree.set_root(root);
        tree.add_child(root, leaf).unwrap();

        if let Some(node) = tree.get_node_mut(leaf) {
            node.name = Some("Leaf_A".to_string());
            let mut props = BTreeMap::new();
            props.insert("color".to_string(), "red".to_string());
            node.properties = Some(props);
        }

        let output = to_forest(&tree, 0.0).unwrap();
        assert!(
            output.contains(r"{\color{red}{Leaf A}},"),
            "expected colored leaf content, got: {}",
            output
        );
        assert!(
            !output.contains(r", \color{red}{Leaf A},"),
            "unexpected leading comma in colored leaf output: {}",
            output
        );
        assert!(
            !output.contains(",,"),
            "unexpected consecutive commas in forest output: {}",
            output
        );
    }

    #[test]
    fn to_forest_replaces_underscore_with_space() {
        let mut tree = Tree::new();
        let root = tree.add_node();
        let leaf = tree.add_node();
        let _ = tree.set_root(root);
        tree.add_child(root, leaf).unwrap();

        tree.get_node_mut(leaf).unwrap().name = Some("Homo_sapiens".to_string());

        let output = to_forest(&tree, 0.0).unwrap();
        assert!(
            output.contains("{Homo sapiens}"),
            "underscore should become space, got: {}",
            output
        );
    }

    #[test]
    fn latex_escape_special_chars() {
        assert_eq!(latex_escape("\\"), r"\textbackslash{}");
        assert_eq!(latex_escape("{"), r"\{");
        assert_eq!(latex_escape("}"), r"\}");
        assert_eq!(latex_escape("#"), r"\#");
        assert_eq!(latex_escape("$"), r"\$");
        assert_eq!(latex_escape("%"), r"\%");
        assert_eq!(latex_escape("&"), r"\&");
        assert_eq!(latex_escape("~"), r"\textasciitilde{}");
        assert_eq!(latex_escape("^"), r"\textasciicircum{}");
        assert_eq!(latex_escape("plain"), "plain");
    }

    #[test]
    fn to_forest_escapes_latex_special_chars() {
        let mut tree = Tree::new();
        let root = tree.add_node();
        let leaf = tree.add_node();
        let _ = tree.set_root(root);
        tree.add_child(root, leaf).unwrap();

        tree.get_node_mut(leaf).unwrap().name = Some(r"A{B}C\D".to_string());
        if let Some(node) = tree.get_node_mut(leaf) {
            let mut props = BTreeMap::new();
            props.insert("label".to_string(), "E%F".to_string());
            props.insert("comment".to_string(), "G&H".to_string());
            node.properties = Some(props);
        }

        let output = to_forest(&tree, 0.0).unwrap();
        assert!(
            output.contains(r"\{"),
            "expected escaped brace, got: {}",
            output
        );
        assert!(
            output.contains(r"\}"),
            "expected escaped brace, got: {}",
            output
        );
        assert!(
            output.contains(r"\textbackslash{}"),
            "expected escaped backslash, got: {}",
            output
        );
        assert!(
            output.contains(r"\%"),
            "expected escaped percent, got: {}",
            output
        );
        assert!(
            output.contains(r"\&"),
            "expected escaped ampersand, got: {}",
            output
        );
        // Raw special characters should not appear in the output.
        assert!(
            !output.contains("{B}"),
            "unescaped brace group in output: {}",
            output
        );
        assert!(
            !output.contains(r"C\D"),
            "unescaped backslash in output: {}",
            output
        );
        assert!(
            !output.contains("E%F"),
            "unescaped percent in output: {}",
            output
        );
        assert!(
            !output.contains("G&H"),
            "unescaped ampersand in output: {}",
            output
        );
    }

    #[test]
    fn to_forest_escapes_visual_property_values() {
        let mut tree = Tree::new();
        let root = tree.add_node();
        let leaf = tree.add_node();
        let _ = tree.set_root(root);
        tree.add_child(root, leaf).unwrap();

        if let Some(node) = tree.get_node_mut(leaf) {
            node.name = Some("Leaf".to_string());
            let mut props = BTreeMap::new();
            props.insert("dot".to_string(), "#FF_00$00".to_string());
            props.insert("bar".to_string(), "25%".to_string());
            props.insert("rec".to_string(), "a&b".to_string());
            props.insert("tri".to_string(), "x\\y".to_string());
            node.properties = Some(props);
        }

        let output = to_forest(&tree, 0.0).unwrap();
        // Underscores become spaces; LaTeX special chars are escaped.
        assert!(
            output.contains("dot={\\#FF 00\\$00}"),
            "expected escaped hash, dollar and space in dot value, got: {}",
            output
        );
        assert!(
            output.contains("bar={25\\%}"),
            "expected escaped percent in bar value, got: {}",
            output
        );
        assert!(
            output.contains("rec={a\\&b}"),
            "expected escaped ampersand in rec value, got: {}",
            output
        );
        assert!(
            output.contains("tri={x\\textbackslash{}y}"),
            "expected escaped backslash in tri value, got: {}",
            output
        );
        // Raw special characters should not appear in the output.
        assert!(
            !output.contains("#FF_00$00"),
            "unescaped dot value in output: {}",
            output
        );
        assert!(
            !output.contains("25%"),
            "unescaped bar value in output: {}",
            output
        );
        assert!(
            !output.contains("a&b"),
            "unescaped rec value in output: {}",
            output
        );
        assert!(
            !output.contains(r"x\y"),
            "unescaped tri value in output: {}",
            output
        );
    }

    #[test]
    fn calc_length_rejects_overflow() {
        let huge_edge = (i32::MAX as f64) * 10.0;
        assert!(calc_length(huge_edge, 1.0).is_err());
    }

    #[test]
    fn calc_length_rejects_non_positive_height() {
        assert!(calc_length(1.0, 0.0).is_err());
        assert!(calc_length(1.0, -1.0).is_err());
    }

    #[test]
    fn calc_length_rejects_non_finite() {
        assert!(calc_length(f64::NAN, 1.0).is_err());
        assert!(calc_length(f64::INFINITY, 1.0).is_err());
    }
}
