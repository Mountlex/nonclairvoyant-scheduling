use std::{iter::FromIterator, ops::Index};

use rand::{distributions::Distribution, prelude::SliceRandom};
use rand_distr::Pareto;

use crate::Gen;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instance {
    pub jobs: Vec<f64>,
}

impl Instance {
    pub fn len(&self) -> usize {
        self.jobs.len()
    }
}

impl From<Vec<f64>> for Instance {
    fn from(lengths: Vec<f64>) -> Self {
        Instance { jobs: lengths }
    }
}

impl FromIterator<f64> for Instance {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
        Instance {
            jobs: iter.into_iter().collect::<Vec<f64>>(),
        }
    }
}

impl Index<usize> for Instance {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.jobs[index]
    }
}

impl<'a> IntoIterator for &'a Instance {
    type Item = &'a f64;
    type IntoIter = std::slice::Iter<'a, f64>;

    fn into_iter(self) -> Self::IntoIter {
        self.jobs.iter()
    }
}

pub struct InstanceGenParams {
    pub length: usize,
    pub alpha: f64,
}

impl Gen<InstanceGenParams> for Instance {
    fn generate(params: &InstanceGenParams) -> Instance {
        let mut rng = rand::thread_rng();
        let dist = Pareto::new(1.0, params.alpha).unwrap();

        let mut jobs: Vec<f64> = dist
            .sample_iter(&mut rng)
            .take(params.length)
            .map(|j| j as f64)
            .collect();

        jobs.shuffle(&mut rng);

        jobs.into()
    }
}

pub fn analyse_instances(instances: &Vec<Instance>) {
    let flat: Vec<f64> = instances
        .iter()
        .flat_map(|instance| instance.jobs.clone())
        .collect();
    println!("Instance Generation Summary:");
    println!("  Mean: {}", mean(&flat).unwrap());
    println!("  StdDev: {}", std_deviation(&flat).unwrap());
    println!(
        "  Max: {}",
        flat.iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    );
}

fn mean(data: &[f64]) -> Option<f64> {
    let sum = data.iter().sum::<f64>() as f64;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f64),
        _ => None,
    }
}

fn std_deviation(data: &[f64]) -> Option<f64> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as f64);

                    diff * diff
                })
                .sum::<f64>()
                / count as f64;

            Some(variance.sqrt())
        }
        _ => None,
    }
}
