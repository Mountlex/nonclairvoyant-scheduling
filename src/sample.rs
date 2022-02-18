use core::f64;
use std::path::PathBuf;

use anyhow::Result;
use csv::Writer;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::Serialize;
use structopt::StructOpt;

use crate::{
    algorithms::{phase_algorithm, preferrential_rr, spt, two_stage_schedule},
    instance::{analyse_instances, Instance, InstanceGenParams, sample_floats, sample_integers},
    job::Job,
    prediction::{InstancePrediction, PredGenParams, ScaledPredGenParams},
    Gen, alg_identical::{pwspt, pts, wdeq},
};

#[derive(Debug, StructOpt)]
pub struct Cli {
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
    Exp3(Exp3Parameters),
}
#[derive(Debug, Serialize)]
struct Entry {
    name: String,
    param: f64,
    sigma: f64,
    opt: f64,
    alg: f64,
}

#[derive(StructOpt, Debug)]
struct Exp1Parameters {
    #[structopt(short = "l", long, default_value = "30000")]
    instance_length: usize,

    #[structopt(short = "n")]
    num_instances: usize,

    #[structopt(short = "p", default_value = "5")]
    num_preds: usize,

    #[structopt(long = "step-sigma")]
    step_sigma: Option<f64>,

    #[structopt(long = "base-sigma")]
    base_sigma: Option<f64>,

    #[structopt(long = "num-sigma", default_value = "10")]
    num_sigmas: i32,

    #[structopt(long)]
    rel_sigma: bool,

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

    #[structopt(long)]
    rel_sigma: bool,

    #[structopt(short = "l", long, default_value = "1000")]
    instance_length: usize,
}

#[derive(StructOpt, Debug)]
struct Exp3Parameters {
    #[structopt(short = "l", long, default_value = "1000")]
    instance_length: usize,

    #[structopt(short = "n")]
    num_instances: usize,

    #[structopt(short)]
    m: usize,

    #[structopt(short, default_value = "1")]
    scale: usize,

    #[structopt(short = "p", default_value = "5")]
    num_preds: usize,

    #[structopt(long = "base-sigma")]
    base_sigma: Option<f64>,

    #[structopt(long = "num-sigma", default_value = "10")]
    num_sigmas: i32,

    #[structopt(short, long = "l-alpha", default_value = "1.1")]
    length_alpha: f64,

    #[structopt(short, long = "w-alpha", default_value = "2.0")]
    weight_alpha: f64,

    #[structopt(short, long = "r-alpha", default_value = "2.0")]
    release_alpha: f64,
}

#[derive(Debug, Serialize)]
struct Exp2Entry {
    name: String,
    param: f64,
    opt: f64,
    alg: f64,
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
                                let sigma = if let Some(step_sigma) = params.step_sigma {
                                    step_sigma * sigma_num as f64
                                } else {
                                    params.base_sigma.unwrap().powi(sigma_num) - 1.0
                                };
                                (0..params.num_preds)
                                    .flat_map(|_| {
                                        let pred: Instance = if params.rel_sigma {
                                            InstancePrediction::generate(&ScaledPredGenParams {
                                                sigma_scale: sigma,
                                                instance: &instance,
                                            })
                                        } else {
                                            InstancePrediction::generate(&PredGenParams {
                                                sigma: sigma,
                                                instance: &instance,
                                            })
                                        };
                                        //let simple_error = SimpleError::compute(&instance, &pred);
                                        //let maxmin_error = MaxMinError::compute(&instance, &pred);
                                        let mut entries = vec![];
                                        [0.1, 0.66].iter().for_each(|lambda| {
                                            entries.push(Entry {
                                                name: format!("PRR"),
                                                param: *lambda,
                                                sigma,
                                                opt,
                                                alg: preferrential_rr(&instance, &pred, *lambda),
                                            });
                                        });

                                        [0.1, 0.66].iter().for_each(|lambda| {
                                            entries.push(Entry {
                                                name: format!("TwoStage"),
                                                param: *lambda,
                                                sigma,
                                                opt,
                                                alg: two_stage_schedule(&instance, &pred, *lambda),
                                            });
                                        });

                                        [0.25, 10.0].iter().for_each(|lambda| {
                                            let pred = pred.clone();
                                            let phase = phase_algorithm(&instance, &pred, *lambda);
                                            entries.push(Entry {
                                                name: format!("Im et al."),
                                                param: *lambda,
                                                sigma,
                                                opt,
                                                alg: phase,
                                            });
                                        });

                                        entries.push(Entry {
                                            name: format!("Round-Robin"),
                                            param: 0.0,
                                            sigma,
                                            opt,
                                            alg: preferrential_rr(&instance, &pred, 1.0),
                                        });
                                        entries
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
                        let mut instances = vec![];
                        (0..params.timesteps + 1)
                            .into_iter()
                            .flat_map(|round| {
                                let pred = create_mean_instance(
                                    &instances,
                                    params.instance_length,
                                    params.alpha,
                                );
                                let instance: Instance = if params.rel_sigma {
                                    InstancePrediction::generate(&ScaledPredGenParams {
                                        sigma_scale: params.sigma,
                                        instance: &ground_truth,
                                    })
                                } else {
                                    InstancePrediction::generate(&PredGenParams {
                                        sigma: params.sigma,
                                        instance: &ground_truth,
                                    })
                                };

                                let opt = spt(&instance);
                                let mut entries = vec![];

                                [0.1, 0.66].iter().for_each(|lambda| {
                                    entries.push(Exp2Entry {
                                        name: format!("PRR"),
                                        param: *lambda,
                                        round,
                                        opt,
                                        alg: preferrential_rr(&instance, &pred, *lambda),
                                    });
                                });

                                [0.1, 0.66].iter().for_each(|lambda| {
                                    entries.push(Exp2Entry {
                                        name: format!("TwoStage"),
                                        param: *lambda,
                                        round,
                                        opt,
                                        alg: two_stage_schedule(&instance, &pred, *lambda),
                                    });
                                });

                                [0.25, 10.0].iter().for_each(|lambda| {
                                    let pred = pred.clone();
                                    let phase = phase_algorithm(&instance, &pred, *lambda);
                                    entries.push(Exp2Entry {
                                        name: format!("Im et al."),
                                        param: *lambda,
                                        round,
                                        opt,
                                        alg: phase,
                                    });
                                });

                                entries.push(Exp2Entry {
                                    name: format!("Round-Robin"),
                                    param: 0.0,
                                    round,
                                    opt,
                                    alg: preferrential_rr(&instance, &pred, 1.0),
                                });

                                instances.push(instance);
                                entries
                            })
                            .collect::<Vec<Exp2Entry>>()
                    })
                    .collect::<Vec<Exp2Entry>>();
                export(&self.output, results)
            },
            Experiments::Exp3(params) => {
                let instance_params = InstanceGenParams {
                    length: params.instance_length,
                    alpha: params.length_alpha,
                };
                let instances: Vec<(Instance, Vec<f64>, Vec<usize>)> = (0..params.num_instances)
                    .map(|_| (Instance::generate(&instance_params), sample_floats(params.weight_alpha, params.instance_length), sample_integers(params.release_alpha, params.instance_length))
                )
                    .collect();
                let results: Vec<Entry> = instances
                    .into_par_iter()
                    .flat_map(|(instance, weights, releases)| {
                        println!("pwspt");
                        let opt = pwspt(&instance, &weights, &releases, params.m, params.scale);
                        println!("wdeq");

                        let wdeq = wdeq(&instance, &weights, &releases, params.m, params.scale);
                        (0..params.num_sigmas)
                            .flat_map(|sigma_num| {
                                let sigma = params.base_sigma.unwrap().powi(sigma_num) - 1.0;
                                (0..params.num_preds)
                                    .flat_map(|_| {
                                        let pred: Instance = InstancePrediction::generate(&PredGenParams {
                                                sigma,
                                                instance: &instance,
                                            });
                                        
                                        let mut entries = vec![];
                                        [0.1, 0.5, 0.8].iter().for_each(|lambda| {
                                        println!("pts");

                                            entries.push(Entry {
                                                name: format!("PTS"),
                                                param: *lambda,
                                                sigma,
                                                opt,
                                                alg: pts(&instance, &pred, &weights, &releases, *lambda, params.m, params.scale),
                                            });
                                        });

                                        entries.push(Entry {
                                            name: format!("WDEQ"),
                                            param: 0.0,
                                            sigma,
                                            opt,
                                            alg: wdeq,
                                        });
                                        entries
                                    })
                                    .collect::<Vec<Entry>>()
                            })
                            .collect::<Vec<Entry>>()
                    })
                    .collect();

                export(&self.output, results)
            }
        }
    }
}

fn create_mean_instance(instances: &[Instance], instance_length: usize, alpha: f64) -> Instance {
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
        let instance_params = InstanceGenParams {
            length: instance_length,
            alpha: alpha,
        };
        Instance::generate(&instance_params)
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
