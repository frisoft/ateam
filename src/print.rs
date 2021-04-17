use super::table;
use super::types::Review;
use super::types::ScoredPr;

pub fn prs(sprs: &[ScoredPr], num: Option<usize>, debug: bool, short: bool, json: bool) {
    let limit = num.unwrap_or(10000);
    if json {
        json_prs(&sprs, limit);
    } else if short {
        short_prs(&sprs, limit, debug);
    } else {
        print!("{}", table::from(&sprs, limit, debug));
    }
}

fn short_prs(sprs: &[ScoredPr], limit: usize, debug: bool) {
    for spr in sprs.iter().take(limit) {
        pr(spr, debug);
    }
}

fn pr(spr: &ScoredPr, _debug: bool) {
    println!("{}", &spr.pr);
}

pub fn reviews(reviews: &[Review], json: bool) {
    if json {
        json_reviews(reviews);
    } else {
        print!("{}", table::from_reviews(reviews));
    }
}

fn json_prs(sprs: &[ScoredPr], limit: usize) {
    let len = sprs.len();
    let l = if limit > len { len } else { limit };
    let j = match serde_json::to_string(&sprs[..l]) {
        Ok(json) => json,
        Err(error) => error.to_string(),
    };
    print!("{}", j);
}

fn json_reviews(reviews: &[Review]) {
    let j = match serde_json::to_string(&reviews) {
        Ok(json) => json,
        Err(error) => error.to_string(),
    };
    print!("{}", j);
}
