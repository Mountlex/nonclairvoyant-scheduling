use algorithms::preferrential_rr;
use job::Job;
use rand::{distributions::Uniform, prelude::Distribution};

use crate::{algorithms::{phase_algorithm, spt}, error::{ErrorMeasure, MaxMinError, SimpleError}};

mod error;
mod job;
mod algorithms;

fn main() {
    let step = Uniform::new(1.0, 100.0);
    let mut rng = rand::thread_rng();
    let lengths: Vec<f64> = step.sample_iter(&mut rng).take(30000).collect();
    let preds: Vec<f64> = step.sample_iter(&mut rng).take(30000).collect();
    let jobs = create_jobs(&lengths, &preds);

    let alg1 = preferrential_rr(jobs.clone(), 0.5);
    let alg2 = phase_algorithm(jobs.clone(), 0.5);
    println!("prr        = {}", alg1);
    println!("phase      = {}", alg2);
    println!("opt        = {}", spt(&lengths));
    println!("simple err = {}", SimpleError::compute(&jobs));
    println!("MaxMin err = {}", MaxMinError::compute(&jobs));
}

fn correct_prediction(lengths: &[f64]) -> Vec<Job> {
    lengths.into_iter().enumerate().map(|(i, p)| Job::new(i, *p, *p)).collect()
}

fn create_jobs(lengths: &[f64], preds: &[f64]) -> Vec<Job> {

    lengths.into_iter().zip(preds.into_iter()).enumerate().map(|(i, (p,y))| Job::new(i, *p, *y)).collect()
}