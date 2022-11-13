use super::table;
use super::types::Review;
use super::types::ScoredPr;

pub fn prs(sprs: &[ScoredPr], num: Option<usize>, debug: bool, short: bool, json: bool) -> String {
    let limit = num.unwrap_or(10000);
    if json {
        json_prs(sprs, limit)
    } else if short {
        short_prs(sprs, limit, debug)
    } else {
        format!("{}", table::from(sprs, limit, debug))
    }
}

fn short_prs(sprs: &[ScoredPr], limit: usize, debug: bool) -> String {
    sprs.iter()
        .take(limit)
        .map(|spr| pr(spr, debug))
        .collect::<Vec<String>>()
        .join("\n")
}

fn pr(spr: &ScoredPr, _debug: bool) -> String {
    format!("{}", &spr.pr)
}

pub fn reviews(reviews: &[Review], json: bool) -> String {
    if json {
        json_reviews(reviews)
    } else {
        format!("{}", table::from_reviews(reviews))
    }
}

fn json_prs(sprs: &[ScoredPr], limit: usize) -> String {
    let len = sprs.len();
    let l = if limit > len { len } else { limit };
    match serde_json::to_string(&sprs[..l]) {
        Ok(json) => json,
        Err(error) => error.to_string(),
    }
}

fn json_reviews(reviews: &[Review]) -> String {
    match serde_json::to_string(&reviews) {
        Ok(json) => json,
        Err(error) => error.to_string(),
    }
}
