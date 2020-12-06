//! The naive auto complete suggestion module


use crate::AutoCompletor;
use crate::Suggestion;
use std::collections::BTreeSet;
use std::rc::Rc;

#[derive(Debug)]
pub struct NaiveAutoComplete {
    suggestions: BTreeSet<Rc<Suggestion>>,
}

impl NaiveAutoComplete {
    pub fn new(suggestions: &[(&str, u32)]) -> Self {
        let s = suggestions
            .iter()
            .map(|(sug, sco)| Rc::new(Suggestion::new(Rc::new((**sug).into()), *sco)))
            .collect::<BTreeSet<Rc<Suggestion>>>();
        NaiveAutoComplete { suggestions: s }
    }

    pub fn suggestions(&self, prefix: &str) -> Vec<Rc<Suggestion>> {
        let matching_suggestions = self
            .suggestions
            .iter()
            .rev()
            .filter(|suggestion| suggestion.word.starts_with(prefix))
            .take(5)
            .map(|s| s.clone())
            .collect::<Vec<Rc<Suggestion>>>();
        matching_suggestions
    }

    
}

impl AutoCompletor for NaiveAutoComplete {
    fn suggestions(&self, prefix: &str) -> Vec<Rc<Suggestion>> { 
        self.suggestions(prefix)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
use crate::Suggestion;

    use super::NaiveAutoComplete;

    #[test]
    fn naive_suggestions_test() {
        let auto_complete = NaiveAutoComplete::new(&[
            ("car", 1),
            ("carpet", 2),
            ("carpenter", 3),
            ("cocoon", 5),
            ("cain", 3),
            ("cameo", 3),
            ("ball", 4),
            ("baller", 5),
        ]);
        assert_eq!(
            suggestions_as_str(&auto_complete.suggestions("car")),
            &["carpenter", "carpet", "car"]
        );
        assert_eq!(
            suggestions_as_str(&auto_complete.suggestions("carp")),
            &["carpenter", "carpet"]
        );
        assert_eq!(
            suggestions_as_str(&auto_complete.suggestions("c")),
            &["cocoon", "carpenter", "cameo", "cain", "carpet"]
        );
        assert_eq!(
            suggestions_as_str(&auto_complete.suggestions("bal")),
            &["baller", "ball"]
        );
        assert_eq!(
            suggestions_as_str(&auto_complete.suggestions("balle")),
            &["baller"]
        );
    }

    fn suggestions_as_str<'a>(suggestions: &'a [Rc<Suggestion>]) -> Vec<&'a str> {
        suggestions
            .iter()
            .map(|b| &b.word)
            .map(|b| &**b)
            .map(|b| &b[..])
            .collect::<Vec<&str>>()
    }
}
