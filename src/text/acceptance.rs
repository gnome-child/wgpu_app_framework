use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use super::{
    Buffer,
    buffer::{Position, Range},
    edit::{Edit, Editor, History},
    unicode,
};

#[test]
fn editing_one_line_preserves_every_other_line_layout_identity() {
    let text = (0..1_000)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut buffer = Buffer::from_multiline_text(text);
    let before = (0..buffer.logical_line_count())
        .map(|line| buffer.line_layout_identity(line).unwrap())
        .collect::<Vec<_>>();
    let edit_line = 500;
    let edit_position = buffer.line_start_offsets()[edit_line] + 2;
    let mut editor = Editor::new();
    let mut state = buffer.initial_state();

    editor.apply_edit(
        &mut buffer,
        &mut state,
        Edit::set_position(Position::new(edit_position)),
    );
    editor.apply_edit(&mut buffer, &mut state, Edit::insert("!"));

    for (line, identity) in before.into_iter().enumerate() {
        let after = buffer.line_layout_identity(line).unwrap();
        if line == edit_line {
            assert_eq!(after.id, identity.id, "the touched line keeps its owner id");
            assert_ne!(
                after.revision, identity.revision,
                "the touched line advertises its changed layout revision"
            );
        } else {
            assert_eq!(
                after, identity,
                "untouched line {line} must stay cache-stable"
            );
        }
    }
}

#[test]
#[ignore = "100k-operation source-span/reference-model acceptance property"]
fn source_span_buffer_matches_reference_through_100k_edit_undo_operations() {
    const OPERATIONS: usize = 100_000;
    const INSERTIONS: &[&str] = &["", "a", "é", "e\u{301}", "👩‍💻", "\n", "beta", "\nλ\n"];

    let mut random = 0xd1b5_4a32_d192_ed03_u64;
    let mut model = String::new();
    let mut undo = Vec::new();
    let mut redo = Vec::new();
    let mut buffer = Buffer::new_multiline();
    let mut state = buffer.initial_state();
    let mut editor = Editor::new();
    let mut history = History::default();

    for operation in 0..OPERATIONS {
        random = next_random(random);
        let choice = (random % 10) as usize;
        if choice < 2 && !undo.is_empty() {
            redo.push(model.clone());
            model = undo.pop().unwrap();
            let outcome = history.undo(&mut buffer, &mut state);
            assert!(!outcome.unavailable, "undo {operation} should be available");
        } else if choice == 2 && !redo.is_empty() {
            undo.push(model.clone());
            model = redo.pop().unwrap();
            let outcome = history.redo(&mut buffer, &mut state);
            assert!(!outcome.unavailable, "redo {operation} should be available");
        } else {
            let boundaries = unicode::source_grapheme_boundaries(&model);
            random = next_random(random);
            let mut start_slot = (random as usize) % boundaries.len();
            random = next_random(random);
            let mut end_slot = (random as usize) % boundaries.len();
            if model.len() > 384 {
                start_slot = start_slot.min(boundaries.len().saturating_sub(2));
                end_slot = (start_slot + 1).max(end_slot);
            }
            if start_slot > end_slot {
                std::mem::swap(&mut start_slot, &mut end_slot);
            }
            let range = boundaries[start_slot]..boundaries[end_slot];
            random = next_random(random);
            let inserted = if model.len() > 384 {
                ""
            } else {
                INSERTIONS[(random as usize) % INSERTIONS.len()]
            };
            let mut next = model.clone();
            next.replace_range(range.clone(), inserted);
            if next != model {
                undo.push(model);
                redo.clear();
                model = next;
                history.apply_edit(
                    &mut editor,
                    &mut buffer,
                    &mut state,
                    Edit::replace_range(Range::from(range), inserted),
                );
            }
        }

        assert_eq!(
            buffer.text(),
            model,
            "byte mismatch after operation {operation}"
        );
        assert_eq!(
            buffer.line_start_offsets().as_ref(),
            &reference_line_starts(&model),
            "line-index mismatch after operation {operation}"
        );
        random = next_random(random);
        let probe = (random as usize) % (model.len() + 1);
        assert_eq!(
            buffer.inner.document.floor_grapheme_boundary(probe),
            unicode::floor_grapheme_boundary(&model, probe),
            "floor grapheme mismatch after operation {operation} at {probe}"
        );
        assert_eq!(
            buffer.inner.document.ceil_grapheme_boundary(probe),
            unicode::ceil_grapheme_boundary(&model, probe),
            "ceil grapheme mismatch after operation {operation} at {probe}"
        );
    }
}

#[test]
#[ignore = "release-mode measured text-buffer acceptance benchmark"]
fn source_span_buffer_meets_load_typing_and_clone_bounds() {
    let path = std::env::temp_dir().join(format!(
        "wgpu_l3_buffer_acceptance_{}.txt",
        std::process::id()
    ));
    let line = "0123456789abcdef\n";
    let contents = line.repeat((8 * 1024 * 1024) / line.len());
    std::fs::write(&path, contents).expect("8 MiB acceptance fixture should write");

    let load = median((0..7).map(|_| {
        let started = Instant::now();
        black_box(Buffer::from_file(&path).expect("acceptance fixture should load"));
        started.elapsed()
    }));
    let typing =
        [10, 2_500_000, 5_000_000, 10_000_000].map(|bytes| (bytes, typing_duration(bytes, 100)));
    let clone_small = clone_duration(10, 100_000);
    let clone_large = clone_duration(10_000_000, 100_000);

    eprintln!("8 MiB load: {:.3} ms", load.as_secs_f64() * 1_000.0);
    for (bytes, elapsed) in typing {
        eprintln!(
            "typing {bytes} bytes: {:.3} us/edit",
            elapsed.as_secs_f64() * 10_000.0
        );
    }
    eprintln!(
        "clone 10 B / 10 MB: {:.3} / {:.3} ns",
        clone_small.as_secs_f64() * 10_000.0,
        clone_large.as_secs_f64() * 10_000.0
    );

    assert!(
        load < Duration::from_millis(100),
        "8 MiB load took {load:?}"
    );
    let min_typing = typing.iter().map(|(_, elapsed)| *elapsed).min().unwrap();
    let max_typing = typing.iter().map(|(_, elapsed)| *elapsed).max().unwrap();
    assert!(
        max_typing.as_secs_f64() / min_typing.as_secs_f64() < 3.0,
        "long-line typing ratio exceeded constant-cost bound: {typing:?}"
    );
    assert!(
        clone_large.as_secs_f64() / clone_small.as_secs_f64() < 2.0,
        "large-buffer clone must stay O(1): small={clone_small:?}, large={clone_large:?}"
    );

    let _ = std::fs::remove_file(path);
}

fn next_random(random: u64) -> u64 {
    random.wrapping_mul(6364136223846793005).wrapping_add(1)
}

fn reference_line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(
            text.bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
        )
        .collect()
}

fn median(values: impl IntoIterator<Item = Duration>) -> Duration {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort_unstable();
    values[values.len() / 2]
}

fn typing_duration(bytes: usize, edits: usize) -> Duration {
    let text = "a".repeat(bytes);
    median((0..5).map(|_| {
        let mut buffer = Buffer::from_multiline_text(text.clone());
        let mut state = buffer.initial_state();
        let mut editor = Editor::new();
        let started = Instant::now();
        for _ in 0..edits {
            black_box(editor.apply_edit(&mut buffer, &mut state, Edit::insert("x")));
        }
        let elapsed = started.elapsed();
        black_box(buffer);
        elapsed
    }))
}

fn clone_duration(bytes: usize, clones: usize) -> Duration {
    let buffer = Buffer::from_multiline_text("a".repeat(bytes));
    median((0..5).map(|_| {
        let started = Instant::now();
        for _ in 0..clones {
            black_box(buffer.clone());
        }
        started.elapsed()
    }))
}
