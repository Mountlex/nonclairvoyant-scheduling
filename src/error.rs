use crate::{algorithms::spt, instance::Instance, job::Job};


pub trait ErrorMeasure {
    fn compute(jobs: &[Job]) -> f64;
}

pub struct SimpleError;

impl ErrorMeasure for SimpleError {
    fn compute(jobs: &[Job]) -> f64 {
        jobs.into_iter().map(|j| (j.pred - j.length).abs()).sum::<f64>() * jobs.len() as f64
    }
}


pub struct MaxMinError;

impl ErrorMeasure for MaxMinError {
    fn compute(jobs: &[Job]) -> f64 {
        let max_length: Instance = jobs.iter().map(|j| j.length.max(j.pred)).collect();
        let min_length: Instance = jobs.iter().map(|j| j.length.min(j.pred)).collect();
        spt(&max_length) - spt(&min_length)
    }
}
