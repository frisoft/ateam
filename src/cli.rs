use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(author, about)]
pub struct Command {
    #[structopt(name = "repository")]
    pub repo: String,
}

pub fn command() -> Command {
    Command::from_args()
}
