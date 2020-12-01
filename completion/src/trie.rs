//! This is the trie module.
//! This module implements trie data structure in a compressed form.

use crate::{internal::Node, Suggestion};
use crate::internal::{Edge};
use std::{rc::Rc};
use std::{
    fmt::{Debug},
};
/// The trie data structure.
/// This is mainly used for getting auto-complete suggestions
#[derive(Debug, PartialEq)]
pub struct Trie {
    root: Node,
}

impl Trie {

    /// Initializes the Trie from a given root `Node`
    fn from(root: Node) -> Self {
        Trie { root }
    }

    /// Initializes the Trie from a given list of suggestions and scores (as a tuple)
    /// The scores are used in ordering the suggestions.
    pub fn new(input: &[(&str, u32)]) -> Self {
        let root = Node::empty();
        let mut trie = Trie::from(root);
        input.iter().for_each(|(input, score)| {
            trie.insert(((*input).into(), *score));
        });
        trie
    }

    /// inserts the given (suggestion, score) tuple into the `Trie`
    pub fn insert(&mut self, suggestion: (String, u32)) {
        let str: Vec<char> = suggestion.0.chars().collect();
        let suggestion = Rc::new(Suggestion::new(Rc::new(suggestion.0), suggestion.1));
        Trie::insert_at(&mut self.root, &str, suggestion);
    }

    fn insert_at(node: &mut Node, remaining: &[char], suggestion: Rc<Suggestion>) {
        node.add_suggestion(suggestion.clone());
        if remaining.len() == 0 {
            return;
        }
        let ch = remaining[0];
        if let Some(mut edge) = node.edges.remove(&ch) {
            let mut part_index: usize = 0;
            let part = &edge.part;
            let part_chars: Vec<char> = part.chars().collect();
            // advance until they match
            // e.g. "car" (part) & "can" (remaining), will result in
            // matching till "ca"
            while part_index < part.len()
                && part_index < remaining.len()
                && part_chars[part_index] == remaining[part_index]
            {
                part_index = part_index + 1;
            }
            //"ca"
            let prefix = &part_chars[..part_index];
            //"r"
            let suffix_of_part = &part_chars[part_index..];
            //"n"
            let suffix_of_remaining = &remaining[part_index..];
            // prefix < part, i.e. we need to branch now
            if prefix.len() < part_chars.len() {
                // create a temporary edge for the common prefix ("ca")
                let mut temp_edge = Edge::empty(prefix.iter().collect());
                temp_edge.node.top_suggestions = edge.node.top_suggestions.clone();
                // insert the suffix of the part ("r")
                temp_edge.node.edges.insert(
                    suffix_of_part[0].into(),
                    Edge::new_node(suffix_of_part.iter().collect(), edge.node),
                );
                // insert the suffix on the remaining input ("n")
                Trie::insert_at(&mut temp_edge.node, suffix_of_remaining, suggestion);
                // insert the temporary edge back into the trie
                node.edges.insert(prefix[0].into(), temp_edge);
            } else {
                Trie::insert_at(&mut edge.node, suffix_of_remaining, suggestion);
                node.edges.insert(ch, edge);
            }
        } else {
            node.edges
                .insert(ch, Edge::new(remaining.iter().collect(), suggestion));
        }
    }

    /// Returns the top suggestions for the give `prefix`
    pub fn suggestions(&self, prefix: &str) -> Vec<Rc<Suggestion>> {
        let inp: Vec<char> = prefix.chars().collect();
        Trie::walk(&self.root, &inp)
    }

    fn walk(node: &Node, input: &[char]) -> Vec<Rc<Suggestion>> {
        if input.len() == 0 {
            return node.sorted_suggestions();
        } else {
            if let Some(edge) = node.edges.get(&input[0]) {
                let mut index = 0;
                let part: Vec<char> = edge.part.chars().collect();
                while index < input.len() && index < part.len() && input[index] == part[index] {
                    index = index + 1
                }
                if index == part.len() {
                    // exhausted all characters in part, we continue walking
                    return Trie::walk(&edge.node, &input[index..]);
                } else if index == input.len() {
                    // exhausted all characters in input, we return this node's suggestion
                    return edge.node.sorted_suggestions();
                } else {
                    // there is a mismatch, no suggestions found.
                    return vec![];
                }
            }
            return vec![];
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::{Edge, Node, Suggestion, Trie};

    #[test]
    fn trie_structural_test() {
        let actual = Trie::new(&[("car", 1), ("carpet", 2), ("carpenter", 3)]);

        let mut root = Node::empty();
        let mut car = new_edge(&mut root.edges, 'c', "car", "car", 1);
        car = car.with_top_suggestions(vec![("car", 1), ("carpet", 2), ("carpenter", 3)]);
        let carpe = empty_edge(&mut car.edges, 'p', "pe");
        carpe.with_top_suggestions(vec![("carpet", 2), ("carpenter", 3)]);

        let edges = &mut carpe.edges;

        let carpet = new_edge(edges, 't', "t", "carpet", 2);
        carpet.suggestion = Some(Rc::new(Suggestion::new(Rc::new("carpet".into()), 2)));
        carpet.with_top_suggestions(vec![("carpet", 2)]);

        let carpenter = new_edge(edges, 'n', "nter", "carpenter", 3);
        carpenter.suggestion = Some(Rc::new(Suggestion::new(Rc::new("carpenter".into()), 3)));
        carpenter.with_top_suggestions(vec![("carpenter", 3)]);

        root.with_top_suggestions(vec![("car", 1), ("carpet", 2), ("carpenter", 3)]);

        let expected = Trie::from(root);
        assert_eq!(dbg!(expected), dbg!(actual));
    }

    #[test]
    fn trie_suggestions_test() {
        let trie = dbg!(Trie::new(&[
            ("car", 1),
            ("carpet", 2),
            ("carpenter", 3),
            ("cocoon", 5),
            ("cain", 3),
            ("cameo", 3),
            ("ball", 4),
            ("baller", 5)
        ]));
        assert_suggestions("car", vec!["carpenter", "carpet", "car"], &trie);
        assert_suggestions("carp", vec!["carpenter", "carpet"], &trie);
        assert_suggestions("carpe", vec!["carpenter", "carpet"], &trie);
        assert_suggestions("carpeo", vec![], &trie);
        assert_suggestions("c", vec!["cocoon", "carpenter", "cameo", "cain", "carpet"], &trie);
        assert_suggestions("carpen", vec!["carpenter"], &trie);
        assert_suggestions("carpet", vec!["carpet"], &trie);
        assert_suggestions("bali", vec![], &trie);
        assert_suggestions("balle", vec!["baller"], &trie);
        assert_suggestions("ball", vec!["baller", "ball"], &trie);
        assert_suggestions("", vec!["cocoon", "baller", "ball", "carpenter", "cameo"], &trie);
    }

    fn assert_suggestions(prefix: &str, expected: Vec<&str>, trie: &Trie) {
        let actual: Vec<String> = trie
            .suggestions(prefix)
            .iter()
            .map(|s| &s.word)
            .map(|p| (**p).clone())
            .collect();
        assert_eq!(
            expected, actual,
            "\nSuggestions for '{}' expected ={:?}, actual ={:?}",
            prefix, &expected, &actual
        );
    }

    fn new_edge<'a>(
        edges: &'a mut HashMap<char, Edge>,
        c: char,
        part: &'a str,
        suggestion: &str,
        score: u32,
    ) -> &'a mut Node {
        &mut edges
            .entry(c)
            .or_insert(Edge::new(
                part.into(),
                Rc::new(Suggestion::new(Rc::new(suggestion.into()), score)),
            ))
            .node
    }

    fn empty_edge<'a>(edges: &'a mut HashMap<char, Edge>, c: char, part: &'a str) -> &'a mut Node {
        &mut edges.entry(c).or_insert(Edge::empty(part.into())).node
    }
}
