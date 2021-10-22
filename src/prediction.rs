use crate::{Gen, instance::Instance};
use rand_distr::{Distribution, Normal};

pub type Prediction = Instance;

pub struct PredGenParams<'a> {
    pub instance: &'a Instance,
    pub sigma: f64
}

impl Gen<PredGenParams<'_>> for Prediction {
    fn generate(params: &PredGenParams) -> Prediction {
        let mut rng = rand::thread_rng();
        
        let preds: Vec<f64> = params.instance.jobs.iter().map(|job| {
            let dist = Normal::new(*job, params.sigma).unwrap();
            let mut p = dist.sample(&mut rng);
            while p < 1.0 {
                p = dist.sample(&mut rng);
            }
            p
        }).collect();
        preds.into()
    }
}

pub struct WCPredGenParams<'a> {
    pub instance: &'a Instance,
}

impl Gen<WCPredGenParams<'_>> for Prediction {
    fn generate(params: &WCPredGenParams) -> Prediction {
        let mut jobs: Vec<(usize, &f64)> = params.instance.jobs.iter().enumerate().collect();
        jobs.sort_by(|(_,a), (_,b)| a.partial_cmp(b).unwrap());
        let (ord, mut jobs): (Vec<usize>, Vec<&f64>) = jobs.into_iter().unzip();
        jobs.reverse();
        let mut preds: Vec<(usize, &f64)> = ord.into_iter().zip(jobs.into_iter()).collect();
        preds.sort_by_key(|(idx,_)| *idx);

        //println!("instance: {:?}", params.instance.jobs);

        let preds: Vec<f64> = preds.into_iter().map(|(_,j)|*j).collect();
        //println!("preds: {:?}", preds);

        preds.into()
    }
}