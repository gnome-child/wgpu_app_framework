#[derive(Debug, Clone)]
pub struct State {
    pub clicks: u32,
    pub wrap: bool,
    pub grid: bool,
    pub mode: Mode,
    pub level: f64,
    pub query: String,
    pub show_advanced: bool,
    pub last_status: String,
    pub record_sort: wgpu_l3::table::SortState,
    pub record_notes: HashMap<u64, String>,
    pub record_counts: HashMap<u64, i64>,
    pub record_enabled: HashMap<u64, bool>,
    pub(super) record_order: Option<RecordOrder>,
    pub expanded_rows: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Design,
    Inspect,
    Preview,
}

impl State {
    pub fn reset(&mut self) {
        *self = Self::default();
        self.last_status = "reset controls".to_owned();
    }
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Design => "Design",
            Self::Inspect => "Inspect",
            Self::Preview => "Preview",
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            clicks: 0,
            wrap: true,
            grid: false,
            mode: Mode::Design,
            level: 42.0,
            query: String::new(),
            show_advanced: true,
            last_status: "ready".to_owned(),
            record_sort: wgpu_l3::table::SortState::new(
                "record",
                wgpu_l3::table::SortDirection::Ascending,
            ),
            record_notes: HashMap::new(),
            record_counts: HashMap::new(),
            record_enabled: HashMap::new(),
            record_order: None,
            expanded_rows: false,
        }
    }
}

impl wgpu_l3::state::State for State {}
use std::collections::HashMap;

pub(super) const RECORD_COUNT: usize = 1_000_000;

#[derive(Debug, Clone)]
pub(super) struct RecordOrder {
    rows: std::rc::Rc<[usize]>,
    indices: std::rc::Rc<[usize]>,
}

impl RecordOrder {
    fn new(
        len: usize,
        column: wgpu_l3::interaction::Id,
        direction: wgpu_l3::table::SortDirection,
        notes: &HashMap<u64, String>,
        counts: &HashMap<u64, i64>,
        enabled: &HashMap<u64, bool>,
    ) -> Option<Self> {
        let mut rows = match column.as_str() {
            "detail" => Self::detail_rows(len, direction),
            "note" => Self::note_rows(len, direction, notes),
            "count" => (0..len).collect(),
            "enabled" => Self::enabled_rows(len, direction, enabled),
            _ => return None,
        };
        if column.as_str() == "count" {
            rows.sort_by(|left, right| {
                let ordering = counts
                    .get(&(*left as u64))
                    .copied()
                    .unwrap_or(0)
                    .cmp(&counts.get(&(*right as u64)).copied().unwrap_or(0));
                match direction {
                    wgpu_l3::table::SortDirection::Ascending => ordering,
                    wgpu_l3::table::SortDirection::Descending => ordering.reverse(),
                }
            });
        }
        let mut indices = vec![0; rows.len()];
        for (index, row) in rows.iter().copied().enumerate() {
            indices[row] = index;
        }
        Some(Self {
            rows: rows.into(),
            indices: indices.into(),
        })
    }

    fn detail_rows(len: usize, direction: wgpu_l3::table::SortDirection) -> Vec<usize> {
        if len == 0 {
            return Vec::new();
        }
        let mut rows = Vec::with_capacity(len);
        rows.push(0);
        if len > 1 {
            let mut current: usize = 1;
            for _ in 1..len {
                rows.push(current);
                if current.saturating_mul(10) < len {
                    current *= 10;
                } else {
                    while current % 10 == 9 || current + 1 >= len {
                        current /= 10;
                    }
                    current += 1;
                }
            }
        }
        if direction == wgpu_l3::table::SortDirection::Descending {
            rows.reverse();
        }
        rows
    }

    fn note_rows(
        len: usize,
        direction: wgpu_l3::table::SortDirection,
        notes: &HashMap<u64, String>,
    ) -> Vec<usize> {
        let mut has_note = vec![false; len];
        let mut nonempty: Vec<_> = notes
            .iter()
            .filter_map(|(key, note)| {
                let key = *key as usize;
                (key < len && !note.is_empty()).then(|| {
                    has_note[key] = true;
                    key
                })
            })
            .collect();
        nonempty.sort_by(|left, right| {
            let ordering = notes[&(*left as u64)].cmp(&notes[&(*right as u64)]);
            let ordering = match direction {
                wgpu_l3::table::SortDirection::Ascending => ordering,
                wgpu_l3::table::SortDirection::Descending => ordering.reverse(),
            };
            ordering.then_with(|| left.cmp(right))
        });
        let empty = (0..len).filter(|row| !has_note[*row]);
        match direction {
            wgpu_l3::table::SortDirection::Ascending => empty.chain(nonempty).collect(),
            wgpu_l3::table::SortDirection::Descending => {
                nonempty.into_iter().chain(empty).collect()
            }
        }
    }

    fn enabled_rows(
        len: usize,
        direction: wgpu_l3::table::SortDirection,
        enabled: &HashMap<u64, bool>,
    ) -> Vec<usize> {
        let first = direction == wgpu_l3::table::SortDirection::Descending;
        let value = |row: usize| enabled.get(&(row as u64)).copied().unwrap_or(row % 2 == 0);
        let mut rows = Vec::with_capacity(len);
        rows.extend((0..len).filter(|row| value(*row) == first));
        rows.extend((0..len).filter(|row| value(*row) != first));
        rows
    }

    pub(super) fn row(&self, index: usize) -> usize {
        self.rows[index]
    }

    pub(super) fn index_of(&self, row: usize) -> Option<usize> {
        self.indices.get(row).copied()
    }
}

impl State {
    pub(super) fn refresh_record_order(&mut self) {
        self.record_order = RecordOrder::new(
            RECORD_COUNT,
            self.record_sort.column(),
            self.record_sort.direction(),
            &self.record_notes,
            &self.record_counts,
            &self.record_enabled,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_record_order_sorts_every_derived_value_without_materializing_records() {
        let notes = HashMap::from([(2, "zulu".to_owned()), (5, "alpha".to_owned())]);
        let counts = HashMap::from([(2, 4), (5, -1)]);
        let enabled_values = HashMap::from([(0, false), (1, true)]);
        let count = RecordOrder::new(
            8,
            wgpu_l3::interaction::Id::new("count"),
            wgpu_l3::table::SortDirection::Ascending,
            &notes,
            &counts,
            &enabled_values,
        )
        .expect("count owns a gallery order");
        assert_eq!(
            (0..8).map(|index| count.row(index)).collect::<Vec<_>>(),
            [5, 0, 1, 3, 4, 6, 7, 2]
        );
        assert_eq!(count.index_of(2), Some(7));

        let enabled_order = RecordOrder::new(
            8,
            wgpu_l3::interaction::Id::new("enabled"),
            wgpu_l3::table::SortDirection::Descending,
            &notes,
            &counts,
            &enabled_values,
        )
        .expect("enabled owns a gallery order");
        assert_eq!(
            (0..8)
                .map(|index| enabled_order.row(index))
                .collect::<Vec<_>>(),
            [1, 2, 4, 6, 0, 3, 5, 7]
        );
        assert_eq!(enabled_order.index_of(0), Some(4));

        let detail = RecordOrder::new(
            25,
            wgpu_l3::interaction::Id::new("detail"),
            wgpu_l3::table::SortDirection::Ascending,
            &notes,
            &counts,
            &enabled_values,
        )
        .expect("detail owns a gallery order");
        assert_eq!(
            &detail.rows[..14],
            [0, 1, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 2, 20]
        );

        let note = RecordOrder::new(
            8,
            wgpu_l3::interaction::Id::new("note"),
            wgpu_l3::table::SortDirection::Descending,
            &notes,
            &counts,
            &enabled_values,
        )
        .expect("note owns a gallery order");
        assert_eq!(
            (0..8).map(|index| note.row(index)).collect::<Vec<_>>(),
            [2, 5, 0, 1, 3, 4, 6, 7]
        );
    }
}
