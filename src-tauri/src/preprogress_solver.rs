use std::sync::Arc;

use ffxiv_crafting::{Skills, Status};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use super::{ProgressSolver, QualitySolver, Solver};

pub struct PreprogressSolver<const MN: usize, const WN: usize>
where
    [[(); WN + 1]; MN + 1]:,
{
    progress_solver: Arc<ProgressSolver<MN, WN>>,
    progress_index: Vec<usize>,
    quality_solvers: Vec<QualitySolver<MN, WN>>,
}

impl<const MN: usize, const WN: usize> PreprogressSolver<MN, WN>
where
    [[(); WN + 1]; MN + 1]:,
{
    pub fn new(
        init_status: Status,
        tail_len: usize,
        progress_solver: Arc<ProgressSolver<MN, WN>>,
        allowed_list: Vec<Skills>,
    ) -> Self {
        let progress_list = progress_solver.possible_progresses();
        let progress_index = progress_list
            .iter()
            .scan(0, |prev, &x| {
                let v = x - *prev;
                *prev = x;
                Some(v)
            })
            .enumerate()
            .map(|(i, v)| std::iter::repeat(i).take(v as usize))
            .flatten()
            .chain(std::iter::once(init_status.recipe.difficulty as usize))
            .collect();

        let quality_solvers = progress_list
            .iter()
            .take(tail_len)
            .map(|v| {
                let mut s = init_status.clone();
                s.progress = s.recipe.difficulty - *v;
                QualitySolver::new(s, progress_solver.clone(), allowed_list.clone())
            })
            .collect();
        Self {
            progress_solver,
            progress_index,
            quality_solvers,
        }
    }
}

impl<const MN: usize, const WN: usize> Solver for PreprogressSolver<MN, WN>
where
    [[(); WN + 1]; MN + 1]:,
{
    fn init(&mut self) {
        self.quality_solvers.par_iter_mut().for_each(|qs| qs.init());
    }

    fn read(&self, s: &Status) -> Option<Skills> {
        let left_progress = s.recipe.difficulty - s.progress;
        let i = self.progress_index[left_progress as usize];
        self.quality_solvers.get(i)?.read(s)
    }

    fn read_all(&self, s: &Status) -> Vec<Skills> {
        let left_progress = s.recipe.difficulty - s.progress;
        let i = self.progress_index[left_progress as usize];
        match self.quality_solvers.get(i) {
            Some(qs) => qs.read_all(s),
            None => vec![],
        }
    }
}
