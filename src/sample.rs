use core::f64;
use std::path::PathBuf;

use anyhow::Result;
use indicatif::ParallelProgressIterator;
use itertools_num::linspace;
use rayon::prelude::*;
use structopt::StructOpt;
use serde::Serialize;
use csv::Writer;

use crate::{Gen, algorithms::{phase_algorithm, preferrential_rr, spt}, error::{ErrorMeasure, MaxMinError, SimpleError}, instance::{Instance, InstanceGenParams, analyse_instances}, job::Job, prediction::{PredGenParams, Prediction}};

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(short = "l", long, default_value = "30000")]
    instance_length: usize,

    #[structopt(short = "n")]
    num_instances: usize,

    #[structopt(short = "p", default_value = "5")]
    num_preds: usize,

    #[structopt(long = "step-sigma", default_value = "50.0")]
    step_sigma: f64,

    #[structopt(long = "num-sigma", default_value = "10")]
    num_sigmas: usize,

    #[structopt(short, long = "alpha", default_value = "1.1")]
    alpha: f64,

    #[structopt(long = "num-lambdas", default_value = "5")]
    num_lambdas: usize,

    

    #[structopt(short, long, parse(from_os_str), default_value = "result.csv")]
    output: PathBuf
}

#[derive(Debug, Serialize)]
struct Entry {
    lambda: f64,
    sigma: f64,
    opt: f64,
    phase: f64,
    prr: f64,
    simple_error: f64,
    maxmin_error: f64,
}

impl Cli {
    pub fn sample(&self) -> Result<()> {
        let instance_params = InstanceGenParams {
            length: self.instance_length,
            alpha: self.alpha
        };
        let instances: Vec<Instance> = (0..self.num_instances).map(|_| 
            Instance::generate(&instance_params)
        ).collect();
        analyse_instances(&instances);
        let results: Vec<Entry> = instances.into_par_iter().progress_count(self.num_instances as u64).flat_map(|instance| {
            let opt = spt(&instance);
            (0..self.num_sigmas).flat_map(|sigma_num| {
                let sigma = self.step_sigma * sigma_num as f64;
                (0..self.num_preds).flat_map(|_| {     
                    let pred_params = PredGenParams {
                        sigma,
                        instance: &instance
                    };
                    let pred = Prediction::generate(&pred_params);
                    let jobs = create_jobs(&instance, &pred);
                    let simple_error = SimpleError::compute(&jobs);
                    let maxmin_error = MaxMinError::compute(&jobs);
                    linspace(0.0, 1.0, self.num_lambdas).map(|lambda| {
                        let pred = pred.clone();
                        let jobs = create_jobs(&instance, &pred);
                        let prr = preferrential_rr(jobs.clone(), lambda);
                        let phase = phase_algorithm(jobs, lambda);


                        Entry {
                            lambda,
                            sigma,
                            opt,
                            phase,
                            prr,
                            simple_error,
                            maxmin_error,
                        }
                    }).collect::<Vec<Entry>>()
                }).collect::<Vec<Entry>>()
            }).collect::<Vec<Entry>>()
        }).collect();

        export(&self.output, results)
    }
}

fn create_jobs(instance: &Instance, pred: &Prediction) -> Vec<Job> {
    instance.into_iter().zip(pred.into_iter()).enumerate().map(|(i, (p,y))| Job::new(i, *p, *y)).collect()
}

fn export(output: &PathBuf, results: Vec<Entry>) -> Result<()> {
    let mut wtr = Writer::from_path(output)?;
    for entry in results {
        wtr.serialize(entry)?;
    }
    Ok(())
}
