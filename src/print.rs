use super::table;
use super::types::ScoredPr;

pub fn prs(sprs: &[ScoredPr], num: Option<usize>, debug: bool, short: bool) {
    let limit = num.unwrap_or(10000);
    if short {
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
