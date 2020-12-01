//! The internal module. This module defines the internal data structures used in the trie
use crate::Suggestion;
use std::{collections::BTreeSet, option::Option, cmp::Ordering, rc::Rc};
use std::{
    collections::HashMap,
    fmt::{Debug, Error},
    result::Result,
};
/// Represents a node in the Trie.
/// A node contains a list of edges (represented as a character to edge map).
/// Each edge is associated with the part of the string.
/// A node contains a map of edges (to nodes) and a flag to indicate a terminal node.
pub(crate) struct Node {
    pub(crate) edges: HashMap<char, Edge>,
    pub(crate) suggestion: Option<Rc<Suggestion>>,
    pub(crate) top_suggestions: BTreeSet<Rc<Suggestion>>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("Node")
            .field("edges", &self.edges)
            .field("suggestion", &self.suggestion)
            .field("top_suggestions", &self.top_suggestions)
            .finish()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        let mut mine: Vec<&Rc<Suggestion>> = self.top_suggestions.iter().collect();
        mine.sort();
        let mut theirs: Vec<&Rc<Suggestion>> = other.top_suggestions.iter().collect();
        theirs.sort();
        self.edges == other.edges && self.suggestion == other.suggestion && mine == theirs
    }
}

impl Node {
    fn new(suggestion: Rc<Suggestion>) -> Self {
        let mut node = Node::empty();
        node.add_suggestion(suggestion.clone());
        node.suggestion = Some(suggestion);
        node
    }

    pub(crate) fn empty() -> Self {
        Node {
            edges: HashMap::new(),
            suggestion: None,
            top_suggestions: BTreeSet::new(),
        }
    }

    pub(crate) fn with_top_suggestions<'a>(
        &'a mut self,
        mut suggestions: Vec<(&str, u32)>,
    ) -> &'a mut Self {
        self.top_suggestions.clear();
        suggestions.drain(0..).for_each(|(w, s)| {
            self.add_suggestion(Rc::new(Suggestion::new(Rc::new(w.into()), s)));
        });
        self
    }

    pub(crate) fn add_suggestion(&mut self, suggestion: Rc<Suggestion>) {
        if self.top_suggestions.len() < 5 {
            // keep the size of the suggestions to 10
            self.top_suggestions.insert(suggestion);
        } else {
            let min_score_suggestion = self.first_suggestion().unwrap().clone();
            // if current min < suggestion score, pop the min and add the sugestion
            // we always want to keep a min heap of top scores
            if suggestion.partial_cmp(&min_score_suggestion) == Some(Ordering::Greater){
                self.top_suggestions.remove(&min_score_suggestion);
                self.top_suggestions.insert(suggestion);
            }
        }
    }

    fn first_suggestion(&self) -> Option<&Rc<Suggestion>> {
        self.top_suggestions.iter().nth(0)
    }

    pub(crate) fn sorted_suggestions(&self) -> Vec<Rc<Suggestion>> {
        self.top_suggestions
            .iter()
            .rev()
            .map(|s| s.clone())
            .collect::<Vec<Rc<Suggestion>>>()
    }
}

/// Represents an Edge in the trie
/// Each edge is associated with the part of the string and another node
#[derive(Debug, PartialEq)]
pub(crate) struct Edge {
    pub(crate) part: String,
    pub(crate) node: Node,
}

impl Edge {
    pub(crate) fn new_node(part: String, node: Node) -> Self {
        Edge { part, node }
    }
    pub(crate) fn new(part: String, suggestion: Rc<Suggestion>) -> Self {
        Edge::new_node(part, Node::new(suggestion))
    }

    pub(crate) fn empty(part: String) -> Self {
        Edge {
            part,
            node: Node::empty(),
        }
    }
}
