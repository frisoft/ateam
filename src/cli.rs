use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(author, about)]
pub struct Command {
    #[structopt(name = "repository")]
    pub repo: String,
    #[structopt(long, short)]
    pub debug: bool,
    #[structopt(long, short)]
    pub num: Option<usize>,
}

pub fn command() -> Command {
    Command::from_args()
}
