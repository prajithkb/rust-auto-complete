//! This is the core library that powers the auto complete.
//! This is essentially a trie that is constructed from a set of sentences. 
#![allow(dead_code)]

use core::cmp::Ordering;
use std::rc::Rc;

mod internal;
pub mod naive;
pub mod trie;

/// Represents a suggestion, i.e. a full word/sentence with an associated score.
/// The score is used to rank the suggestions (higher score = higher suggestion)
#[derive(Debug, Eq, Hash)]
pub struct Suggestion {
    pub word: Rc<String>,
    score: u32,
}

pub trait AutoCompletor {
    fn suggestions(&self, prefix: &str) -> Vec<Rc<Suggestion>>;
}

impl Suggestion {
    pub(crate) fn new(word: Rc<String>, score: u32) -> Self {
        Suggestion { word, score }
    }
}

impl Ord for Suggestion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then_with(|| self.word.cmp(&other.word))
    }
}

impl PartialEq for Suggestion {
    fn eq(&self, other: &Self) -> bool {
        self.score == self.score && self.word == other.word 
    }
}

impl PartialOrd for Suggestion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.score
            .cmp(&other.score)
            .then_with(|| self.word.cmp(&other.word)))
    }
}



#[cfg(test)]
mod tests {

    use crate::{naive::NaiveAutoComplete, trie::Trie};
    #[test]
    fn trie_vs_naive_test() {
        let data = [
            ("car", 1),
            ("carpet", 2),
            ("carpenter", 3),
            ("cocoon", 5),
            ("cain", 2),
            ("aba", 3),
            ("acas", 4),
            ("ballcdcder", 5),
            ("caa", 5),
            ("cascasin", 3),
            ("cacs", 3),
            ("bascascll", 4),
            ("basller", 5),
            ("cdacs", 3),
            ("dascascll", 4),
            ("dasller", 5),
            ("eeacs", 3),
            ("escascll", 4),
            ("eesller", 5),
        ];
        let prefixes = ["c", "a", "d", "e", "ca", "da", "es", "ba", "ac", "cd"];
        let trie = dbg!(Trie::new(&data));
        let naive =dbg!(NaiveAutoComplete::new(&data));
        
        prefixes
            .iter()
            .map(|&prefix| (prefix, trie.suggestions(prefix), naive.suggestions(prefix)))
            .for_each(|(prefix, trie, naive)| assert_eq!(trie, naive, "\nPrefix: {}\ntrie suggestions: {:#?} != naive suggestions:{:#?} ", prefix, trie, naive));


    }

}