use std::any::TypeId;

use super::{Command, Registry, Spec};

pub struct Set {
    pub(in crate::command) entries: Vec<Entry>,
}

pub(in crate::command) struct Entry {
    command_type: TypeId,
    command_name: &'static str,
    pub(in crate::command) spec: Spec,
    pub(in crate::command) install: fn(&mut Registry, Spec),
}

#[derive(Clone, Copy)]
pub struct Member<'a> {
    command_name: &'static str,
    spec: &'a Spec,
}

impl Set {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn include<C: Command>(mut self, spec: Spec) -> Self {
        let command_type = TypeId::of::<C>();
        self.entries
            .retain(|entry| entry.command_type != command_type);
        self.entries.push(Entry {
            command_type,
            command_name: C::NAME,
            spec,
            install: install::<C>,
        });
        self
    }

    pub fn without<C: Command>(mut self) -> Self {
        let command_type = TypeId::of::<C>();
        self.entries
            .retain(|entry| entry.command_type != command_type);
        self
    }

    pub fn members(&self) -> impl ExactSizeIterator<Item = Member<'_>> {
        self.entries.iter().map(|entry| Member {
            command_name: entry.command_name,
            spec: &entry.spec,
        })
    }
}

impl Default for Set {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Member<'a> {
    pub fn command_name(self) -> &'static str {
        self.command_name
    }

    pub fn spec(self) -> &'a Spec {
        self.spec
    }
}

fn install<C: Command>(registry: &mut Registry, spec: Spec) {
    registry.register::<C>(spec);
}
