# Command Module Organization

This document captures the organization scheme used by `src/command`.

The command module is organized around one rule:

```text
Central concepts are re-exported.
Supporting concepts stay namespaced.
Internal routing details stay private.
```

## Public Shape

Callsites should import central concepts from the framework root or from `command`:

```rust
use wgpu_l3::{Command, Registry, Response, State, Target};
use wgpu_l3::command::{Call, Effect};
```

They should not reach through implementation modules for central concepts:

```rust
// Avoid.
// Import central concepts from `command` or the crate root, not from implementation modules.
```

Supporting concepts remain behind their module namespace:

```rust
command::call::Context
command::call::Scope
command::call::Source
command::definition::Definition
command::shortcut::Shortcut
command::target::Kind
command::binding::Route
```

That keeps the main API readable while preserving context around narrower concepts.

## Module Map

`command::Command`

The command contract. It lives at `command` module root because it is the central concept, not a support type. A command is a type-level description: args, output, name, display text, hint, repeatability, and target kind.

`command::Target`

The execution contract for a specific command on a specific runtime target. Implementing `Target<C>` says: this target knows how to execute command `C`.

`command::Registry`

Stores command definitions, shortcuts, configured state, running state, and typed call preparation. The registry does not decide focus or scope by itself; runtime/app layers provide context.
Registering a command definition does not make it available in every context.
Unprojected state is treated as unavailable for presentation, but request preparation only rejects explicit disabled/running state, target-contract mismatches, unknown commands, and invalid args. Final execution still validates the concrete target state.

`command::Call`

A typed request to invoke a command. It carries typed args plus requested target kind, source, scope, window, origin, and repeat state.

`command::Response`

The result of invoking a command. It carries command output and follow-up effects. A response can pipe output into another command call or attach runtime/task effects.

`command::Effect`

Follow-up work produced by a response: runtime effect, batch, command call, or task.

`command::State`

Runtime command affordance for a context: available, active, running, display override, and hint override.

`command::Args`

Controlled command invocation arguments. Standard arg types implement conversion to and from `args::Raw`, with validation such as string size limits.

`command::Output`

Marker trait for command output values.

## Namespaced Support Modules

`command::args`

Defines `Raw`, `Kind`, and argument validation errors. This is the boundary for external/raw command arguments.

`command::binding`

Defines command-facing binding support: `Route`, `Binding`, and `Responder`. A binding says what command route a widget or responder exposes; it is not command execution.

UI does not store these command types directly. Command-backed widgets project them into generic action metadata with `.action()`, and the app command bridge decodes that action metadata back into command routes when it resolves registry state or builds a command request. Menu items follow the same rule: `widget::menu::Item` stores `action::Route`, while its command-facing builders project command routes immediately. The reverse conversion is runtime/internal; callers should keep naming commands by Rust type.

`command::call`

Defines invocation context vocabulary: `Context`, `Scope`, `Source`, erased `Any`, internal raw calls, and typed `Invocation<C>`.

`command::definition`

Defines `Definition`, `Contract`, and the erased function pointers used by registration. This is where static command metadata is attached to runtime invocation machinery.

`command::shortcut`

Defines keyboard shortcut matching and display formatting.

`command::target`

Defines target capability vocabulary. `target::Kind` is a runtime category used for routing and resolution. `target::Category` lets command definitions advertise trait-style target categories.

## Private Internals

`command::key`

Internal routing identity for command types. It is intentionally not public API. Callers name commands by Rust type, not by an exposed command id.

`command::output`

Small internal module for the `Output` marker trait. The trait itself is re-exported as `command::Output`.

`command::response`

Implementation module for `Response`. The type is re-exported as `command::Response`.

## Typed And Erased Boundary

The public API is typed:

```rust
Call::<Save>::new::<Editor>(())
```

The runtime needs erased storage and dispatch:

```text
Call<C> -> call::Any -> definition invoker -> Target<C>::invoke
```

The erased layer is private to the command runtime. It exists so the registry and app runtime can queue, validate, and route calls without exposing command ids or forcing callsites to handle dynamic typing.

## Execution Flow

```text
Command type defines contract
Registry stores Definition
UI/app creates or resolves Call<C>
Runtime resolves target from scope/focus/capture/window/path
Runtime/app resolves whether the call has a handler path
Registry validates target contract, configured state, and args
Registry invokes Target<C>
Target returns Response<Output>
Runtime handles Response effects
```

The target decides behavior. The runtime decides where the command runs.

## Adding A Command

Use the macro for ordinary command definitions:

```rust
command!(pub Save {
    name: "save",
    display: "Save",
    hint: "Write the current document",
    repeatable: false,
});
```

Then implement behavior on a target:

```rust
impl command::Target<Save> for Editor {
    fn state(&self, context: &command::call::Context) -> command::State {
        command::State::available_if(self.is_dirty())
    }

    fn invoke(
        &mut self,
        _args: (),
        invocation: command::call::Invocation<Save>,
    ) -> command::Response<()> {
        self.save(invocation.context());
        command::Response::none()
    }
}
```

Register the definition with the target implementation:

```rust
registry.define::<Save, Editor>(|definition| {
    definition.shortcut(command::shortcut::Shortcut::ctrl('s'))
});
```

## Boundary Rules

- Command definitions describe invocation and effect contracts.
- Target implementations execute commands.
- Registry stores definitions and context-scoped state.
- Availability is runtime state, not a side effect of definition registration.
- Runtime/app layers resolve scope, focus, capture, and concrete target selection.
- Widgets may project command bindings into generic UI action metadata, but UI should not own command execution or command registry state.
- Command keys are internal routing details, not caller-facing ids.
- If a concept is meant to be used everywhere, re-export it from `command`.
- If a concept only makes sense with its module context, keep it namespaced.
