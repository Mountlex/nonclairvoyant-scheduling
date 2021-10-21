
#[derive(Copy, Clone, Debug, Default)]
pub struct Job {
    pub length: f64,
    pub pred: f64,
    pub completed: bool,
    id: usize,
}

impl Job {
    pub fn new(id: usize, length: f64, pred: f64) -> Self {
        Job {
            id, length, pred, completed: false
        }
    }
}

pub struct Environment {
    pub time: f64,
    pub obj: f64,
    pub n: usize,
    pub jobs: Vec<Job>,
}

impl Environment {
    pub fn new(jobs: Vec<Job>) -> Self {
        Environment {
            time: 0.0, obj: 0.0, n:jobs.len(), jobs
        }
    }

    pub fn nk(&self) -> usize {
        self.jobs.len()
    }

    pub fn process(&mut self, job_idx: usize, amount: f64) -> bool {
        if let Some(job) = self.jobs.get_mut(job_idx) {
            job.length -= amount;
            job.pred = (job.pred - amount).max(0.0);
            if job.length < 0.0 {
                panic!("job length < 0! {}", job.length)
            }
            if job.length == 0.0 {
                job.completed = true;
                self.obj += self.time;
                return true;
            }
            false
        } else {
            panic!("Job not found")
        }
    }

    pub fn complete(&mut self, job_idx: usize) {
        if let Some(job) = self.jobs.get_mut(job_idx) {
            job.completed = true;
            job.length = 0.0;
            self.obj += self.time;          
        }
    }

    pub fn clear_completed(&mut self) {
        self.jobs.retain(|j| !j.completed)
    }

    pub fn run_for(&mut self, time: f64) {
        self.time += time;
    }
}