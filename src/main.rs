use sample::Cli;
use anyhow::Result;

mod error;
mod job;
mod algorithms;
mod instance;
mod prediction;
mod sample;

pub trait Gen<P> {
    fn generate(params: &P) -> Self;
}

#[paw::main]
fn main(args: Cli) -> Result<()> {
    args.sample()
}