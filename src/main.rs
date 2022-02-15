use anyhow::Result;
use sample::Cli;

mod algorithms;
mod error;
mod instance;
mod job;
mod prediction;
mod sample;
mod alg_identical;

pub trait Gen<P> {
    fn generate(params: &P) -> Self;
}

#[paw::main]
fn main(args: Cli) -> Result<()> {
    args.sample()
}
