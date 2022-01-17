use crate::{Gen, instance::Instance};
use rand_distr::{Distribution, Normal};

pub type InstancePrediction = Instance;

#[derive(Clone, Debug, PartialEq)]
pub struct PredGenParams<'a> {
    pub instance: &'a Instance,
    pub sigma: f64
}

impl Gen<PredGenParams<'_>> for InstancePrediction {
    fn generate(params: &PredGenParams) -> InstancePrediction {
        let mut rng = rand::thread_rng();
        
        let preds: Vec<f64> = params.instance.jobs.iter().map(|job| {
            let dist = Normal::new(0.0, params.sigma).unwrap();
            let mut p = dist.sample(&mut rng) + *job;
            while p < 1.0 {
                p = dist.sample(&mut rng) + *job;
            }
            p
        }).collect();
        preds.into()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScaledPredGenParams<'a> {
    pub instance: &'a Instance,
    pub sigma_scale: f64
}

impl Gen<ScaledPredGenParams<'_>> for InstancePrediction {
    fn generate(params: &ScaledPredGenParams) -> InstancePrediction {
        let mut rng = rand::thread_rng();
        
        let preds: Vec<f64> = params.instance.jobs.iter().map(|job| {
            let dist = Normal::new(0.0, job.sqrt() * params.sigma_scale).unwrap();
            let mut p = dist.sample(&mut rng) + *job;
            while p < 1.0 {
                p = dist.sample(&mut rng) + *job;
            }
            p
        }).collect();
        preds.into()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PermutationPrediction {
    pub permutation: Vec<usize>
}


impl Gen<PredGenParams<'_>> for PermutationPrediction {
    fn generate(params: &PredGenParams) -> PermutationPrediction {
        let mut rng = rand::thread_rng();
        
        let preds: Vec<f64> = params.instance.jobs.iter().map(|job| {
            let dist = Normal::new(*job, params.sigma).unwrap();
            let mut p = dist.sample(&mut rng);
            while p < 1.0 {
                p = dist.sample(&mut rng);
            }
            p
        }).collect();
        
        let mut permutation: Vec<(usize, f64)> = preds.into_iter().enumerate().collect();
        permutation.sort_by(|(_,a), (_,b)| a.partial_cmp(b).unwrap());
        PermutationPrediction {
            permutation: permutation.into_iter().map(|(i, _)| i).collect()
        }
    }
}