use crate::{
    algorithms::spt,
    instance::Instance,
    prediction::{InstancePrediction, PermutationPrediction},
};

pub trait ErrorMeasure<P> {
    fn compute(instance: &Instance, pred: &P) -> f64;
}

pub struct SimpleError;

impl ErrorMeasure<InstancePrediction> for SimpleError {
    fn compute(instance: &Instance, pred: &InstancePrediction) -> f64 {
        instance
            .jobs
            .iter()
            .zip(pred.jobs.iter())
            .map(|(p, y)| (*p - *y).abs())
            .sum::<f64>()
            * instance.len() as f64
    }
}

pub struct MaxMinError;

impl ErrorMeasure<InstancePrediction> for MaxMinError {
    fn compute(instance: &Instance, pred: &InstancePrediction) -> f64 {
        let max_length: Instance = instance
            .jobs
            .iter()
            .zip(pred.jobs.iter())
            .map(|(p, y)| (*p).max(*y))
            .collect();
        let min_length: Instance = instance
            .jobs
            .iter()
            .zip(pred.jobs.iter())
            .map(|(p, y)| (*p).min(*y))
            .collect();
        spt(&max_length) - spt(&min_length)
    }
}

pub struct InversionError;

impl ErrorMeasure<PermutationPrediction> for InversionError {
    fn compute(instance: &Instance, pred: &PermutationPrediction) -> f64 {
        let mut permutation: Vec<(usize, usize)> =
            pred.permutation.iter().copied().enumerate().collect();
        permutation.sort_by_key(|(_, j)| *j);
        let job_to_pos: Vec<usize> = permutation.into_iter().map(|(i, _)| i).collect();
        let mut error = 0.0;
        for i in 0..instance.len() {
            for j in 0..instance.len() {
                if (instance[i] < instance[j] || (instance[i] == instance[j] && i < j))
                    && job_to_pos[i] > job_to_pos[j]
                {
                    error += instance[j] - instance[i];
                }
            }
        }
        error
    }
}
