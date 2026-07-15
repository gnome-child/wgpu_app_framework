use std::collections::VecDeque;

pub(super) const SAMPLE_LIMIT: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Samples {
    values: VecDeque<u128>,
    limit: usize,
}

impl Samples {
    pub(super) fn record(&mut self, value: u128) {
        if self.values.len() == self.limit {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    pub(super) fn len(&self) -> usize {
        self.values.len()
    }

    pub(super) fn p50(&self) -> u128 {
        self.percentile(50)
    }

    pub(super) fn p95(&self) -> u128 {
        self.percentile(95)
    }

    pub(super) fn p99(&self) -> u128 {
        self.percentile(99)
    }

    pub(super) fn max(&self) -> u128 {
        self.values.iter().copied().max().unwrap_or(0)
    }

    fn percentile(&self, percentile: usize) -> u128 {
        if self.values.is_empty() {
            return 0;
        }

        let mut sorted = self.values.iter().copied().collect::<Vec<_>>();
        sorted.sort_unstable();
        let rank = ((sorted.len() * percentile).div_ceil(100)).saturating_sub(1);
        sorted[rank.min(sorted.len() - 1)]
    }
}

impl Default for Samples {
    fn default() -> Self {
        Self {
            values: VecDeque::with_capacity(SAMPLE_LIMIT),
            limit: SAMPLE_LIMIT,
        }
    }
}
