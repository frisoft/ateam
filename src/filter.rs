use super::types::ScoredPr;
use regex::Regex;

pub fn regex<'a>(regex_text: &'a Option<String>, prs: Vec<ScoredPr<'a>>) -> Vec<ScoredPr<'a>> {
    if let Some(regex_text) = regex_text {
        let re = Regex::new(regex_text).unwrap();
        prs.into_iter()
            .filter(|pr| re.is_match(&pr.pr.title))
            .collect()
    } else {
        prs
    }
}
