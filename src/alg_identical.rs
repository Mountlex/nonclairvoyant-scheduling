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

    loop {
        // interval [t,t+1]
        let released_jobs: Vec<ID> = releases.iter().enumerate().filter(|(_, &r)| r * scale == t).map(|(id, _)| id).collect();
        if !released_jobs.is_empty() {
            for j in released_jobs {
                jobs.push(Job { id: j, weight: weights[j], pred: instance[j] * scale as f64, length: instance[j] * scale as f64});
            }
        }

        // WDEQ
        let mut rm = m;
        let mut K: Vec<usize> = (0..jobs.len()).collect();
        'find: loop {
            let wk = total_weight(&jobs, &K);
            let mut job: Option<usize> = None;
            for &j in &K {
                if jobs[j].weight * rm as f64 / wk >= 1.0 {
                    jobs[j].length -= 1.0;
                    rm -= 1;
                    job = Some(j.clone());
                    break
                }
            }
            if let Some(job) = job {
                K.retain(|&j| j != job);
            } else {
                break 'find;
            }
        }
        let wk = total_weight(&jobs, &K);
        for j in K {
            let rate = jobs[j].weight * rm as f64 / wk;
            jobs[j].length -= rate;
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

    loop {
        // interval [t,t+1]
        let released_jobs: Vec<ID> = releases.iter().enumerate().filter(|(_, &r)| r * scale == t).map(|(id, _)| id).collect();
        if !released_jobs.is_empty() {
            for j in released_jobs {
                jobs.push(Job { id: j, weight: weights[j], pred: pred[j] * scale as f64, length: instance[j] * scale as f64});
            }
            jobs.sort_by(|j1,j2| (j2.weight / j2.pred).partial_cmp(&(j1.weight / j1.pred)).unwrap());
        }

        // P-WSPT
        let pwspt: Vec<&mut Job> = jobs.iter_mut().take(m).collect();
        for j in pwspt {
            j.length -= 1.0 - robustification
        }

        // WDEQ
        let mut rm = m;
        let mut K: Vec<usize> = (0..jobs.len()).collect();
        'find: loop {
            let wk = total_weight(&jobs, &K);
            let mut job: Option<usize> = None;
            for &j in &K {
                if jobs[j].weight * rm as f64 / wk >= 1.0 {
                    jobs[j].length -= robustification;
                    rm -= 1;
                    job = Some(j.clone());
                    break
                }
            }
            if let Some(job) = job {
                K.retain(|&j| j != job);
            } else {
                break 'find;
            }
        }
        let wk = total_weight(&jobs, &K);
        for j in K {
            let rate = jobs[j].weight * rm as f64 / wk;
            jobs[j].length -= rate * robustification;
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

fn total_weight(jobs: &[Job], indices: &[usize]) -> f64 {
    indices.iter().map(|&i| jobs[i].weight).sum()
}




