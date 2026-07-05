use std::collections::{HashMap, VecDeque};

use super::super::document::{Align, Block, Document, Run, TextDirection, Weight};
use super::key::{BoundsKey, finite_bits};
use super::{Measure, Metrics};

#[derive(Debug)]
pub(super) struct MeasureCache {
    entries: HashMap<MeasureKey, Metrics>,
    order: VecDeque<MeasureKey>,
    capacity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MeasureKey {
    blocks: Vec<BlockKey>,
    max: Option<BoundsKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BlockKey {
    align: Align,
    runs: Vec<RunKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RunKey {
    text: String,
    size: u32,
    weight: Weight,
    direction: TextDirection,
}

impl MeasureCache {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            capacity,
        }
    }

    pub(super) fn get(&self, document: &Document, measure: Measure) -> Option<Metrics> {
        self.entries
            .get(&MeasureKey::new(document, measure))
            .copied()
    }

    pub(super) fn insert(&mut self, document: &Document, measure: Measure, metrics: Metrics) {
        if self.capacity == 0 {
            return;
        }

        let key = MeasureKey::new(document, measure);
        if let Some(entry) = self.entries.get_mut(&key) {
            *entry = metrics;
            return;
        }

        while self.entries.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            } else {
                break;
            }
        }

        self.order.push_back(key.clone());
        self.entries.insert(key, metrics);
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}

impl MeasureKey {
    fn new(document: &Document, measure: Measure) -> Self {
        Self {
            blocks: document
                .blocks()
                .iter()
                .filter(|block| !block.is_empty())
                .map(BlockKey::new)
                .collect(),
            max: measure.max().map(BoundsKey::new),
        }
    }
}

impl BlockKey {
    fn new(block: &Block) -> Self {
        Self {
            align: block.align(),
            runs: block.runs().iter().map(RunKey::new).collect(),
        }
    }
}

impl RunKey {
    fn new(run: &Run) -> Self {
        let style = run.style();

        Self {
            text: run.text().to_owned(),
            size: finite_bits(style.size().max(1.0)),
            weight: style.weight(),
            direction: style.direction(),
        }
    }
}
