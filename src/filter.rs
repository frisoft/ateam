use super::types::ScoredPr;
use regex::Regex;

pub fn regex(regex_text: &Option<String>, prs: Vec<ScoredPr>) -> Vec<ScoredPr> {
    if let Some(regex_text) = regex_text {
        let re = Regex::new(regex_text).unwrap();
        prs.into_iter()
            .filter(|pr| re.is_match(&pr.pr.title))
            .collect()
    } else {
        prs
    }
}
