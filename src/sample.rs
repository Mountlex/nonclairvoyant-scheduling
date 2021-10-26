use core::f64;
use std::path::PathBuf;

use anyhow::Result;
use csv::Writer;
use indicatif::ParallelProgressIterator;
use itertools_num::linspace;
use rand_distr::Distribution;
use rayon::prelude::*;
use serde::Serialize;
use structopt::StructOpt;

use rand::distributions::Uniform;

use crate::{
    algorithms::{phase_algorithm, preferrential_rr, spt},
    error::{ErrorMeasure, MaxMinError, SimpleError},
    instance::{analyse_instances, Instance, InstanceGenParams},
    job::Job,
    prediction::{InstancePrediction, PredGenParams},
    Gen,
};

#[derive(Debug, StructOpt)]
pub struct Cli {  

    #[structopt(long = "num-lambdas", default_value = "5")]
    num_lambdas: usize,

    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "result.csv",
        global = true
    )]
    output: PathBuf,

    #[structopt(subcommand)]
    experiment: Experiments,
}

#[derive(StructOpt, Debug)]
enum Experiments {
    Exp1(Exp1Parameters),
    Exp2(Exp2Parameters),
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

#[derive(StructOpt, Debug)]
struct Exp1Parameters {
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
}

#[derive(StructOpt, Debug)]
struct Exp2Parameters {
    #[structopt(short = "n", default_value = "1")]
    trials: usize,

    #[structopt(short = "t", default_value = "20")]
    timesteps: usize,

    #[structopt(short, long = "alpha", default_value = "1.1")]
    alpha: f64,

    #[structopt(short, long = "sigma", default_value = "1.0")]
    sigma: f64,

    #[structopt(short = "l", long, default_value = "1000")]
    instance_length: usize,
}

#[derive(Debug, Serialize)]
struct Exp2Entry {
    lambda: f64,
    opt: f64,
    prr: f64,
    round: usize,
}

impl Cli {
    pub fn sample(&self) -> Result<()> {
        match &self.experiment {
            Experiments::Exp1(params) => {
                let instance_params = InstanceGenParams {
                    length: params.instance_length,
                    alpha: params.alpha,
                };
                let instances: Vec<Instance> = (0..params.num_instances)
                    .map(|_| Instance::generate(&instance_params))
                    .collect();
                analyse_instances(&instances);
                let results: Vec<Entry> = instances
                    .into_par_iter()
                    .progress_count(params.num_instances as u64)
                    .flat_map(|instance| {
                        let opt = spt(&instance);
                        (0..params.num_sigmas)
                            .flat_map(|sigma_num| {
                                let sigma = params.step_sigma * sigma_num as f64;
                                (0..params.num_preds)
                                    .flat_map(|_| {
                                        let pred_params = PredGenParams {
                                            sigma,
                                            instance: &instance,
                                        };
                                        let pred = InstancePrediction::generate(&pred_params);
                                        let simple_error = SimpleError::compute(&instance, &pred);
                                        let maxmin_error = MaxMinError::compute(&instance, &pred);
                                        linspace(0.0, 1.0, self.num_lambdas)
                                            .map(|lambda| {
                                                let pred = pred.clone();
                                                let prr =
                                                    preferrential_rr(&instance, &pred, lambda);
                                                let phase =
                                                    phase_algorithm(&instance, &pred, lambda);

                                                Entry {
                                                    lambda,
                                                    sigma,
                                                    opt,
                                                    phase,
                                                    prr,
                                                    simple_error,
                                                    maxmin_error,
                                                }
                                            })
                                            .collect::<Vec<Entry>>()
                                    })
                                    .collect::<Vec<Entry>>()
                            })
                            .collect::<Vec<Entry>>()
                    })
                    .collect();

                export(&self.output, results)
            }
            Experiments::Exp2(params) => {
                let results = (0..params.trials)
                    .into_par_iter()
                    .progress_count(params.trials as u64)
                    .flat_map(|_| {
                        let instance_params = InstanceGenParams {
                            length: params.instance_length,
                            alpha: params.alpha,
                        };
                        let ground_truth: Instance = Instance::generate(&instance_params);
                        let pred_params = PredGenParams {
                            sigma: params.sigma,
                            instance: &ground_truth,
                        };
                        let mut instances = vec![];
                        (0..params.timesteps)
                            .into_iter()
                            .flat_map(|round| {
                                let pred = create_mean_instance(&instances, params.instance_length);
                                let instance = InstancePrediction::generate(&pred_params);
                                
                                let opt = spt(&instance);
                                let entries = linspace(0.0, 1.0, self.num_lambdas)
                                    .map(|lambda| {
                                        let prr = preferrential_rr(&instance, &pred, lambda);
                                        //  let phase = phase_algorithm(&instance, &pred, lambda);
                                        Exp2Entry {
                                            lambda,
                                            opt,
                                            prr,
                                            round,
                                        }
                                    })
                                    .collect::<Vec<Exp2Entry>>();
                                instances.push(instance);
                                entries
                            })
                            .collect::<Vec<Exp2Entry>>()
                    })
                    .collect::<Vec<Exp2Entry>>();
                export(&self.output, results)
            }
        }
    }
}

fn create_mean_instance(instances: &[Instance], instance_length: usize) -> Instance {
    if instances.len() > 0 {
        let mut lengths: Vec<f64> = Vec::with_capacity(instances.first().unwrap().len());
        for i in 0..instances.first().unwrap().len() {
            let p =
                instances.iter().map(|instance| instance[i]).sum::<f64>() / instances.len() as f64;
            if p < 1.0 {
                panic!()
            }
            lengths.push(p);
        }
        Instance { jobs: lengths }
    } else {
        let mut rng = rand::thread_rng();
        
        Instance {      
            jobs: Uniform::new(1.0, 100.0)
            .sample_iter(&mut rng)
            .take(instance_length)
            .map(|j| j as f64)
            .collect(),
        }
    }
}

pub fn create_jobs(instance: &Instance, pred: &InstancePrediction) -> Vec<Job> {
    instance
        .into_iter()
        .zip(pred.into_iter())
        .enumerate()
        .map(|(i, (p, y))| Job::new(i, *p, *y))
        .collect()
}

fn export<E: Serialize>(output: &PathBuf, results: Vec<E>) -> Result<()> {
    let mut wtr = Writer::from_path(output)?;
    for entry in results {
        wtr.serialize(entry)?;
    }
    Ok(())
}
