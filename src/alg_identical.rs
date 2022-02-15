use crate::{instance::Instance, prediction::InstancePrediction};


type ID = usize;

#[derive(Clone, Copy, Debug)]
struct Job {
    id: ID,
    weight: f64,
    pred: f64,
    length: f64,
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Job {}

pub fn pwspt(
    instance: &Instance,
    weights: &[f64],
    releases: &[usize],
    m: usize,
    scale: usize,
) -> f64 {
    let mut t: usize = 0;
    let mut obj: usize = 0;

    let mut n = releases.len();
    let mut jobs: Vec<Job> = vec![];

    loop {
        // interval [t,t+1]
        let released_jobs: Vec<ID> = releases.iter().enumerate().filter(|(_, &r)| r * scale == t).map(|(id, _)| id).collect();
        if !released_jobs.is_empty() {
            for j in released_jobs {
                jobs.push(Job { id: j, weight: weights[j], pred: instance[j] * scale as f64, length: instance[j] * scale as f64});
            }
            jobs.sort_by(|j1,j2| (j2.weight / j2.pred).partial_cmp(&(j1.weight / j1.pred)).unwrap());
        }

        // P-WSPT
        let pwspt: Vec<&mut Job> = jobs.iter_mut().take(m).collect();
        for j in pwspt {
            j.length -= 1.0
        }

        t += 1;
        
        // clear finished jobs
        let n_finished = jobs.iter().filter(|j| j.length <= 0.0).count();
        n -= n_finished;
        obj += t * n_finished;
        jobs.retain(|j| j.length > 0.0);

        if n == 0 {
            return obj as f64 / scale as f64
        }
    }
}

pub fn wdeq(
    instance: &Instance,
    weights: &[f64],
    releases: &[usize],
    m: usize,
    scale: usize,
) -> f64 {
    let mut t: usize = 0;
    let mut obj: usize = 0;

    let mut n = releases.len();
    let mut jobs: Vec<Job> = vec![];

    let mut wdeq_rates: Vec<f64> = vec![];
    let mut recompute_rates = true;

    loop {
        // interval [t,t+1]
        let released_jobs: Vec<ID> = releases.iter().enumerate().filter(|(_, &r)| r * scale == t).map(|(id, _)| id).collect();
        if !released_jobs.is_empty() {
            for j in released_jobs {
                jobs.push(Job { id: j, weight: weights[j], pred: instance[j] * scale as f64, length: instance[j] * scale as f64});
            }
            recompute_rates = true;
        }

        if recompute_rates {
            wdeq_rates = compute_wdeq_rates(&jobs, m);
            recompute_rates = false;
        }
        for (idx, j) in jobs.iter_mut().enumerate() {
            j.length -= wdeq_rates[idx];
        }

        t += 1;
        
        // clear finished jobs
        let n_finished = jobs.iter().filter(|j| j.length <= 0.0).count();
        if n_finished > 0 {
            recompute_rates = true;
        }
        n -= n_finished;
        obj += t * n_finished;
        jobs.retain(|j| j.length > 0.0);

        if n == 0 {
            return obj as f64 / scale as f64
        }
    }
}

fn compute_wdeq_rates(jobs: &[Job], m: usize) -> Vec<f64> {
    let mut rm = m;
    let mut rem_jobs: Vec<usize> = (0..jobs.len()).collect();
    let n = jobs.len();
    let mut rates: Vec<f64> = vec![0.0;n];
    'find: loop {
        let wk = total_weight(&jobs, &rem_jobs);
        let mut job: Option<usize> = None;
        for &j in &rem_jobs {
            if jobs[j].weight * rm as f64 / wk >= 1.0 {
                rates[j] = 1.0;
                rm -= 1;
                job = Some(j.clone());
                break
            }
        }
        if let Some(job) = job {
            rem_jobs.retain(|&j| j != job);
        } else {
            break 'find;
        }
    }
    let wk = total_weight(&jobs, &rem_jobs);
    for j in rem_jobs {
        rates[j] = jobs[j].weight * rm as f64 / wk;
    }
    rates
}

pub fn pts(
    instance: &Instance,
    pred: &InstancePrediction,
    weights: &[f64],
    releases: &[usize],
    robustification: f64,
    m : usize,
    scale: usize,
) -> f64 {
    let mut t: usize = 0;
    let mut obj: usize = 0;

    let mut n = releases.len();
    let mut jobs: Vec<Job> = vec![];

    let mut wdeq_rates: Vec<f64> = vec![];
    let mut recompute_rates = true;

    loop {
        // interval [t,t+1]
        let released_jobs: Vec<ID> = releases.iter().enumerate().filter(|(_, &r)| r * scale == t).map(|(id, _)| id).collect();
        if !released_jobs.is_empty() {
            for j in released_jobs {
                jobs.push(Job { id: j, weight: weights[j], pred: pred[j] * scale as f64, length: instance[j] * scale as f64});
            }
            jobs.sort_by(|j1,j2| (j2.weight / j2.pred).partial_cmp(&(j1.weight / j1.pred)).unwrap());
            recompute_rates = true;
        }

        // P-WSPT
        let pwspt: Vec<&mut Job> = jobs.iter_mut().take(m).collect();
        for j in pwspt {
            j.length -= 1.0 - robustification
        }

        // WDEQ
        if recompute_rates {
            wdeq_rates = compute_wdeq_rates(&jobs, m);
            recompute_rates = false;
        }
        for (idx, j) in jobs.iter_mut().enumerate() {
            j.length -= robustification * wdeq_rates[idx];
        }

        t += 1;
        
        // clear finished jobs
        let n_finished = jobs.iter().filter(|j| j.length <= 0.0).count();
        if n_finished > 0 {
            recompute_rates = true;
        }
        n -= n_finished;
        obj += t * n_finished;
        jobs.retain(|j| j.length > 0.0);

        if n == 0 {
            return obj as f64 / scale as f64
        }
    }
}

fn total_weight(jobs: &[Job], indices: &[usize]) -> f64 {
    indices.iter().map(|&i| jobs[i].weight).sum()
}




