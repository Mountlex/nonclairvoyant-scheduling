use rand::prelude::SliceRandom;

use crate::{
    instance::Instance, job::Environment, prediction::InstancePrediction, sample::create_jobs,
};

pub fn spt(instance: &Instance) -> f64 {
    let mut jobs = instance.jobs.clone();
    jobs.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    let mut obj = 0.0;
    let mut t = 0.0;
    for j in jobs {
        t += j;
        obj += t;
    }
    obj
}

pub fn preferrential_rr(
    instance: &Instance,
    pred: &InstancePrediction,
    robustification: f64,
) -> f64 {
    instance.jobs.iter().for_each(|p| assert!(*p >= 1.0));
    let mut jobs = create_jobs(&instance, &pred);
    jobs.sort_by(|j1, j2| j1.length.partial_cmp(&j2.length).unwrap());

    let mut pred_order_help: Vec<(usize, f64)> =
        jobs.iter().enumerate().map(|(i, j)| (i, j.pred)).collect();
    pred_order_help.sort_by(|(_, p1), (_, p2)| p1.partial_cmp(&p2).unwrap());
    let pred_order: Vec<usize> = pred_order_help.into_iter().map(|(i, _)| i).collect();

    let mut n_alive = jobs.len();
    let mut pspt: usize = 0;
    let mut rr: usize = 0;
    let mut t: f64 = 0.0;
    let mut obj: f64 = 0.0;

    while n_alive > 0 {
        if jobs[rr].length <= 0.0 {
            if jobs[rr].completed == false && t > 0.0 {
                panic!("Job length 0 but not finished!")
            }
            rr += 1;
            continue;
        }
        if jobs[pred_order[pspt]].length <= 0.0 {
            pspt += 1;
            continue;
        }

        let l;
        if robustification > 0.0 && robustification < 1.0 {
            l = (jobs[rr].length * (n_alive as f64) / robustification).min(
                (jobs[pred_order[pspt]].length * (n_alive as f64))
                    / ((n_alive as f64) - (n_alive as f64 - 1.0) * robustification),
            );
        } else if robustification == 0.0 {
            l = jobs[pred_order[pspt]].length
        } else {
            l = jobs[rr].length * (n_alive as f64);
        }
        t += l;

        assert!(l >= 0.0);

        let pre_n_alive = n_alive;

        if robustification > 0.0 {
            for (i, job) in jobs.iter_mut().enumerate().skip(rr) {
                if robustification == 1.0 || (i != pred_order[pspt]) {
                    job.length -= l * robustification / (pre_n_alive as f64);
                    if job.length <= 0.0 {
                        //println!("rr completed ");
                        if job.completed == false {
                            job.completed = true;
                            n_alive -= 1;
                            obj += t;
                        }
                        if i == rr {
                            rr += 1;
                        }
                    }
                }
            }
        }

        if robustification < 1.0 {
            if robustification == 0.0 {
                jobs[pred_order[pspt]].length -= l;
            } else {
                jobs[pred_order[pspt]].length -=
                    l * ((1.0 - robustification) + (robustification / (pre_n_alive as f64)));
            }
            if jobs[pred_order[pspt]].length <= 0.0 {
                if jobs[pred_order[pspt]].completed == false {
                    jobs[pred_order[pspt]].completed = true;
                    n_alive -= 1;
                    obj += t;
                }
                pspt += 1;
            }
        }
    }

    return obj;
}

pub fn two_stage_schedule(instance: &Instance, pred: &InstancePrediction, lambda: f64) -> f64 {
    let mut jobs = create_jobs(&instance, &pred);
    jobs.sort_by(|j1, j2| j1.length.partial_cmp(&j2.length).unwrap());

    let opt_y = spt(pred);

    let mut n_alive = jobs.len();
    let mut rr: usize = 0;
    let mut t: f64 = 0.0;
    let mut obj: f64 = 0.0;

    let mut misprediction_detected = false;

    let max_rr =
        lambda * instance.len() as f64 * opt_y / num_integer::binomial(instance.len(), 2) as f64;

    // RR until time == max_rr
    while n_alive > 0 && !misprediction_detected && t < max_rr {
        assert!(rr < jobs.len());
        if jobs[rr].length <= 0.0 {
            if jobs[rr].completed == false && t > 0.0 {
                panic!("Job length 0 but not finished!")
            }
            rr += 1;
            continue;
        }

        let l = (jobs[rr].length * (n_alive as f64)).min(max_rr - t);
        t += l;
        let pre_n_alive = n_alive;
        for (i, job) in jobs.iter_mut().enumerate().skip(rr) {
            job.length -= l / (pre_n_alive as f64);
            if job.length <= 0.0 {
                if job.completed == false {
                    job.completed = true;
                    n_alive -= 1;
                    obj += t;

                    if instance[job.id] != pred[job.id] {
                        misprediction_detected = true;
                    }
                }
                if i == rr {
                    rr += 1;
                }
            }
        }
    }

    assert!(t <= max_rr || !misprediction_detected);

    // Schedule remaining jobs in predicted order; break if a misprediction is being detected.
    jobs.sort_by(|j1, j2| j1.pred.partial_cmp(&j2.pred).unwrap());
    let mut idx = 0;
    while !misprediction_detected && idx < jobs.len() {
        if !jobs[idx].completed {
            t += jobs[idx].length;
            obj += t;
            jobs[idx].length = 0.0;
            jobs[idx].completed = true;
            n_alive -= 1;
            if instance[jobs[idx].id] != pred[jobs[idx].id] {
                misprediction_detected = true;
            }
        }
        idx += 1;
    }

    // RR until all jobs finish.
    jobs.sort_by(|j1, j2| j1.length.partial_cmp(&j2.length).unwrap());
    rr = 0;
    while n_alive > 0 {
        assert!(rr < jobs.len());
        if jobs[rr].length <= 0.0 {
            if jobs[rr].completed == false && t > 0.0 {
                panic!("Job length 0 but not finished!")
            }
            rr += 1;
            continue;
        }

        let l = jobs[rr].length * (n_alive as f64);
        t += l;
        let pre_n_alive = n_alive;
        for (i, job) in jobs.iter_mut().enumerate().skip(rr) {
            job.length -= l / (pre_n_alive as f64);
            if job.length <= 0.0 {
                if job.completed == false {
                    job.completed = true;
                    n_alive -= 1;
                    obj += t;
                }
                if i == rr {
                    rr += 1;
                }
            }
        }
    }

    obj
}

pub fn phase_algorithm(instance: &Instance, pred: &InstancePrediction, epsilon: f64) -> f64 {
    let jobs = create_jobs(&instance, &pred);

    let mut env = Environment::new(jobs);
    let delta = 1.0 / 50.0;

    // line 2
    while env.nk() as f64 >= (env.n as f64).log2() / (epsilon * epsilon * epsilon) {
        // line 3:
        let mk = median_est(&mut env, delta);

        // line 4:
        let error = error_est(&mut env, epsilon, mk);

        // line 5:
        if error >= (epsilon * (delta * delta) * mk * env.nk() as f64 * env.nk() as f64) / 16.0 {
            //line 6:
            env.jobs
                .sort_by(|j1, j2| j1.length.partial_cmp(&j2.length).unwrap());
            let mut rr_per_job = 0.0;
            let mut finished = 0;
            for j in 0..env.nk() {
                if env.process(j, rr_per_job) {
                    finished += 1;
                } else {
                    let amount = env.jobs[j].length.min(2.0 * mk - rr_per_job);
                    let l = amount * (env.nk() - finished) as f64;
                    env.run_for(l);
                    if env.process(j, amount) {
                        finished += 1;
                    }
                    rr_per_job += amount;
                }
            }
            env.clear_completed();
        } else {
            //line 8:
            env.jobs
                .sort_by(|j1, j2| j1.pred.partial_cmp(&j2.pred).unwrap());
            for j in 0..env.nk() {
                if env.jobs[j].pred <= (1.0 + epsilon) * mk {
                    let l = env.jobs[j]
                        .length
                        .min(env.jobs[j].pred + 3.0 * epsilon * mk);
                    env.run_for(l);
                    env.process(j, l);
                }
            }
            env.clear_completed();
        }
    }

    // line 10:
    env.jobs
        .sort_by(|j1, j2| j1.length.partial_cmp(&j2.length).unwrap());

    let mut rr_per_job = 0.0;
    let mut finished = 0;
    for j in 0..env.nk() {
        if env.process(j, rr_per_job) {
            finished += 1;
        } else {
            let amount = env.jobs[j].length;
            let l = amount * (env.nk() - finished) as f64;
            env.run_for(l);
            env.complete(j);
            finished += 1;
            rr_per_job += amount;
        }
    }
    env.clear_completed();
    assert_eq!(env.nk(), 0);
    return env.obj;
}

fn median_est(env: &mut Environment, delta: f64) -> f64 {
    // line 1:
    let sample_size = ((2.0 * env.n as f64).ln() / (delta * delta)).ceil() as usize;
    let indices = (0..env.nk()).collect::<Vec<usize>>();
    let mut sample: Vec<&usize> = sample_with_replacement(&indices, sample_size);
    let max_job_index = **sample.iter().max().unwrap();

    // how often does a job occur in sample
    let mut occurences = vec![0; max_job_index + 1];
    for &&idx in &sample {
        occurences[idx] += 1;
    }

    // sort the jobs in the sample in the order of their completion by RR
    sample.sort_by(|&i, &j| {
        (env.jobs[*i].length / occurences[*i] as f64)
            .partial_cmp(&(env.jobs[*j].length / occurences[*j] as f64))
            .unwrap()
    });

    // remove duplicatd
    sample.dedup();

    let initial_lengths: Vec<f64> = sample.iter().map(|j| env.jobs[**j].length).collect();

    // line 2:
    let mut rr_per_job = 0.0;

    // count completed job for break condition
    let mut finished = 0;

    for (i, &job_idx) in sample.into_iter().enumerate() {
        if env.process(job_idx, rr_per_job * occurences[job_idx] as f64) {
            finished += occurences[job_idx];
        } else {
            let amount = env.jobs[job_idx].length;
            let l = amount * (sample_size - finished) as f64 / occurences[job_idx] as f64;
            env.run_for(l);
            env.complete(job_idx);
            finished += occurences[job_idx];
            rr_per_job += amount / occurences[job_idx] as f64;
        }

        if 2 * finished >= sample_size {
            // line 3
            env.clear_completed();
            return initial_lengths[i];
        }
    }

    panic!("Median estimation did not work properly!");
}

fn error_est(env: &mut Environment, epsilon: f64, est_median: f64) -> f64 {
    // line 1:
    let sample_size = ((env.n as f64).log2() / (epsilon * epsilon)).ceil() as usize;
    let mut indices = Vec::<(usize, usize)>::new();
    for i in 0..env.nk() {
        for j in i..env.nk() {
            indices.push((i, j));
        }
    }
    let sample = sample_with_replacement(&indices, sample_size);
    let mut job_sample: Vec<usize> = sample
        .iter()
        .flat_map(|(i, j)| vec![*i, *j].into_iter())
        .collect();
    job_sample.sort();
    job_sample.dedup();

    // line 2:
    let mut d = vec![0.0; env.nk() + 1];
    let max_l = (1.0 + epsilon) * est_median;
    for job_idx in job_sample {
        let l = env.jobs[job_idx].length.min(max_l);
        d[job_idx] = (l - env.jobs[job_idx].pred.min(max_l)).abs();
        env.run_for(l);
        env.process(job_idx, l);
    }
    env.clear_completed();

    // line 3:
    (indices.len() as f64)
        * sample
            .into_iter()
            .map(|(i, j)| d[*i].min(d[*j]))
            .sum::<f64>()
        / (sample_size as f64)
}

fn sample_with_replacement<T>(set: &[T], sample_size: usize) -> Vec<&T> {
    if set.is_empty() {
        return vec![];
    }
    let mut rng = rand::thread_rng();
    let mut index_sample = Vec::<&T>::with_capacity(sample_size);
    for _ in 0..sample_size {
        let index = set.choose(&mut rng).unwrap();
        index_sample.push(index);
    }
    return index_sample;
}
