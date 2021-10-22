use algorithms::preferrential_rr;
use job::Job;
use rand::{distributions::Uniform, prelude::Distribution};
use sample::Cli;
use anyhow::Result;

use crate::{algorithms::{phase_algorithm, spt}, error::{ErrorMeasure, MaxMinError, SimpleError}};

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