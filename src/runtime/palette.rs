use std::{any::TypeId, cmp::Ordering};

use super::{Runtime, fuzzy, services::Services, transaction};
use crate::{
    command, context as command_context, error::Error, input, interaction, responder, response,
    session, state, subject, view, window,
};

const PAGE_SIZE: usize = 8;

#[derive(Clone)]
struct Match {
    command: command::ResolvedAction,
    score: fuzzy::Score,
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn command_palette_projection(
        &mut self,
        window: window::Id,
    ) -> Option<view::CommandPalette> {
        let query = self.session.command_palette_query(window)?;
        let selected = self
            .session
            .command_palette_selected(window)
            .unwrap_or_default();
        let entries = self.command_palette_matches(window, &query);
        let captured_focus = self
            .session
            .command_palette_captured_focus(window)
            .flatten();
        let subject_path = self
            .composition
            .get(window)
            .map(|composition| composition.subject_path_for_focus(captured_focus))
            .unwrap_or_else(subject::Path::application);
        let selected = selected.min(entries.len().saturating_sub(1));
        let entries = entries
            .into_iter()
            .map(|entry| {
                let command = entry.command;
                let section = section_for(command.claim().provenance(), &subject_path);
                view::CommandPaletteEntry::new(
                    command.trigger(),
                    command
                        .state()
                        .label
                        .clone()
                        .unwrap_or_else(|| command.command_name().to_owned()),
                    section,
                )
            })
            .collect();

        Some(view::CommandPalette::new(
            query,
            selected,
            entries,
            self.active_theme().command_palette().max_results_height(),
        ))
    }

    pub(in crate::runtime) fn handle_command_palette_scope_key(
        &mut self,
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        let scope = self
            .session
            .command_scope(window, self.session.focused(window));
        if scope.kind() != responder::Kind::Transient {
            return Ok(None);
        }
        let Some(query) = self.session.command_palette_query(window) else {
            return Ok(None);
        };
        if modifiers.control() || modifiers.alt() || modifiers.super_key() || modifiers.shift() {
            return Ok(None);
        }

        let entries = self.command_palette_matches(window, &query);
        let len = entries.len();
        let changed = match key {
            input::Key::ArrowDown => self.session.select_command_palette_next(window, len),
            input::Key::ArrowUp => self.session.select_command_palette_previous(window, len),
            input::Key::PageDown => self
                .session
                .select_command_palette_page_next(window, len, PAGE_SIZE),
            input::Key::PageUp => self
                .session
                .select_command_palette_page_previous(window, len, PAGE_SIZE),
            input::Key::Enter => return self.activate_command_palette_selection(window).map(Some),
            _ => return Ok(None),
        };

        if changed {
            self.reveal_command_palette_selection(window, len);
            Ok(Some(self.window_outcome(
                window,
                false,
                response::Effect::Paint,
            )))
        } else {
            Ok(Some(input::Outcome::handled(false, response::Effect::None)))
        }
    }

    pub(in crate::runtime) fn activate_command_palette_binding(
        &mut self,
        window: window::Id,
        binding: &view::Binding,
    ) -> std::result::Result<response::Effect, Error> {
        let focus = self
            .session
            .command_palette_captured_focus(window)
            .flatten();
        let closed = self.session.close_command_palette(window);
        let focused_text = self.prepare_focused_text_for_command(window, binding.command_type())?;
        let effect = self.activate_with_focus(focus, Some(window), binding)?;
        let effect = focused_text
            .committed()
            .map(|outcome| outcome.effect().clone())
            .unwrap_or(response::Effect::None)
            .then(effect)
            .then(if closed {
                response::Effect::Rebuild
            } else {
                response::Effect::None
            });
        Ok(effect)
    }

    fn activate_command_palette_selection(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(query) = self.session.command_palette_query(window) else {
            return Ok(input::Outcome::ignored());
        };
        let entries = self.command_palette_matches(window, &query);
        let selected = self
            .session
            .command_palette_selected(window)
            .unwrap_or_default()
            .min(entries.len().saturating_sub(1));
        let Some(entry) = entries.get(selected).cloned() else {
            return Ok(self.window_outcome(window, false, response::Effect::None));
        };
        let command_type = entry.command.command_type();
        let command_name = entry.command.command_name();
        let history_group = entry.command.history_group();
        let focus = self
            .session
            .command_palette_captured_focus(window)
            .flatten();
        let closed = self.session.close_command_palette(window);
        let focused_text = self.prepare_focused_text_for_command(window, command_type)?;

        let Some(transaction) = self.transact_any_command(
            transaction::AnyInvocation {
                focus,
                window: Some(window),
                command_type,
                command_name,
                history_group,
                source: command_context::Source::Palette,
            },
            |registry, chain, cx| {
                Ok(registry.invoke_any(command_type, command_name, Box::new(()), chain, cx))
            },
        )?
        else {
            let effect = if closed {
                response::Effect::Rebuild
            } else {
                response::Effect::None
            };
            return Ok(self.window_outcome(window, false, effect));
        };

        let changed = focused_text
            .committed()
            .is_some_and(input::Outcome::changed_state)
            || transaction.changed_state;
        let effect = focused_text
            .committed()
            .map(|outcome| outcome.effect().clone())
            .unwrap_or(response::Effect::None)
            .then(transaction.effect)
            .then(if closed {
                response::Effect::Rebuild
            } else {
                response::Effect::None
            });

        transaction
            .response
            .into_result()
            .map(|_| self.window_outcome(window, changed, effect))
    }

    fn command_palette_matches(&mut self, window: window::Id, query: &str) -> Vec<Match> {
        let Some(scope) = self.session.command_palette_captured_scope(window) else {
            return Vec::new();
        };
        let cx = command_context::Context::with_clipboard_source(
            &mut self.clipboard,
            command_context::Source::Palette,
        );
        let services = Services::new(
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            Some(window),
            scope,
        );
        let path = responder::Path::single(scope);
        let mut chain = self
            .responders
            .chain_for_path(&mut self.store, &path, responder::Traversal::Task)
            .with_service(services);

        let population = self.registry.population();
        let candidates = population.palette_candidates();
        let mut entries = population
            .resolve_claimed(candidates, &mut chain, &cx)
            .into_iter()
            .filter(|command| command.state().is_enabled())
            .filter(|command| {
                !(command.listing() == command::Listing::Describer
                    && command.command_type() == TypeId::of::<session::OpenCommandPalette>())
            })
            .filter_map(|command| {
                score_command(query, &command).map(|score| Match { command, score })
            })
            .collect::<Vec<_>>();

        entries.sort_by(compare_matches);
        entries
    }

    fn reveal_command_palette_selection(&mut self, window: window::Id, len: usize) -> bool {
        if len == 0 {
            return false;
        }

        self.session
            .reveal_active_descendant(window, interaction::CommandPalette::results_target())
    }
}

fn score_command(query: &str, command: &command::ResolvedAction) -> Option<fuzzy::Score> {
    let label = command
        .state()
        .label
        .as_deref()
        .unwrap_or(command.command_name());
    [
        fuzzy::score(query, label),
        fuzzy::score(query, command.command_name()),
    ]
    .into_iter()
    .flatten()
    .max()
}

fn compare_matches(left: &Match, right: &Match) -> Ordering {
    left.command
        .claim()
        .provenance()
        .sort_key()
        .cmp(&right.command.claim().provenance().sort_key())
        .then_with(|| right.score.get().cmp(&left.score.get()))
        .then_with(|| {
            left.command
                .registration_index()
                .cmp(&right.command.registration_index())
        })
}

fn section_for(provenance: &responder::Provenance, path: &subject::Path) -> String {
    match provenance.kind() {
        responder::Kind::Captured | responder::Kind::Transient | responder::Kind::Focused => path
            .nearest()
            .map(|segment| segment.label().to_owned())
            .unwrap_or_else(|| subject::Segment::application().label().to_owned()),
        responder::Kind::Ancestor => path
            .nearest_at(provenance.sort_key().1.saturating_add(1))
            .or_else(|| path.nearest())
            .map(|segment| segment.label().to_owned())
            .unwrap_or_else(|| subject::Segment::application().label().to_owned()),
        responder::Kind::Window => subject::Segment::window().label().to_owned(),
        responder::Kind::Workspace | responder::Kind::App => {
            subject::Segment::application().label().to_owned()
        }
        responder::Kind::Framework => subject::Segment::system().label().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_preserves_registration_order_for_ties() {
        assert_eq!(fuzzy::score("", "Undo"), fuzzy::score("", "Redo"));
    }
}
