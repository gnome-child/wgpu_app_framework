# Command Module Organization

This document applies `docs/master_design.md` to the current command,
responder, target, and response modules.

The command system is organized around one rule:

```text
Commands describe what can be asked.
Targets execute commands.
Responders decide where a command can run.
Responses report what happened.
Runtime orchestrates the route.
```

The registry does not know focus, widgets, platform input, or application
intent. It stores command contracts and asks the responder chain for resolved
capability.

## Public Shape

Central command concepts are re-exported from `command`:

```rust
command::Command
command::History
command::HistoryGroup
command::KeyChord
command::Listing
command::Member
command::Observation
command::Registry
command::Set
command::Spec
command::Standard
command::State
command::Trigger
```

The related execution concepts live in their own root modules because they
answer different questions:

```rust
target::Target      // this value can execute this command
responder::Builder  // these are the available command scopes
responder::Chain    // nearest-to-outermost runtime resolution
response::Response  // command output plus side-effect metadata
context::Context    // invocation environment
```

Do not collapse these into a single command object. That was the old hidden
assumption: command identity, execution target, routing scope, and result were
treated as one thing. They are separate concepts.

## Module Map

`command::Command`

The type-level command contract. App code dispatches by Rust type, while
`Command::NAME` provides stable metadata for keymaps, debugging, settings, and
future plugin surfaces. A command declares its argument type, output type,
history policy, and optional history grouping.

`command::Spec`

Registration-time presentation metadata: display label, semantic shortcut, and
`Listing` policy for command-describing surfaces.
Shortcut declarations stay semantic (`Primary+S`, `Standard::Save`) and are
resolved through the active keymap profile at match and presentation time.
`Spec::new`, `.shortcut(...)`, `.key_chord(...)`, and `.listing(...)` are public
because apps register commands through `Runtime::commands(...)`.

`command::Set` and `command::Member`

A `Set` is an app-owned, enumerable bundle of typed command registrations.
Including the same command again replaces its earlier member, and `without`
narrows the bundle before registration. `Member` exposes each command's stable
name and `Spec` for inspection without exposing the registry's erased dispatch
storage.

`command::Registry`

Stores registered command types, specs, shortcut bindings, and registration
order. It can resolve a typed command state, enumerate unit commands for the
palette, and invoke a command after a responder claim has been found. It does
not choose focus or mutate app state by itself.

`command::State`

Resolved affordance for a command in a concrete context: enabled, disabled, or
hidden, with optional checked state, label override, shortcut override, and
tooltip. Availability is runtime state, not a property of registration.
State constructors and modifiers are public because app targets must be able to
report availability without reaching into framework internals.

`command::Trigger`

A typed invocation request carrying command arguments. Widgets and runtime
helpers produce triggers; the registry converts them to erased internal form
only at the routing boundary.

`command::Observation`

Post-command observation context. Observers can react to command outputs and
effects without becoming the command target.

`target::Target<C>`

The execution contract. Implementing `Target<C>` for a value says that value can
report state for `C` and invoke `C`. Target implementations own behavior; they
return `response::Response<C::Output>`.

`responder`

Responder builders describe the available command scopes: app, object,
focused object, and framework services. Runtime builds a nearest-first chain
from these responders plus services. Claim provenance is routing and diagnostic
data; user-facing palette labels come from subject ancestry.

`context::Context`

Carries the invocation source and the narrow capabilities available during one
state query or invocation. Runtime supplies those capabilities; Context does
not own their engines. In particular, visual text motion crosses as the
text-owned `text::selection::CaretMap` contract rather than a concrete UI
layout service. Clipboard and task access retain their own result and absence
semantics.

`response`

Response owns command output, changed-state reporting, and follow-up effects.
Runtime consumes responses to update history, invalidation depth, tasks,
requests, and observations.
Response constructors and result accessors are public because `target::Target`
implementations return responses directly.

## Execution Flow

```text
Command type declares contract
Runtime registers command Spec in Registry
App/framework registers Target implementations in responder scopes
UI/input produces a Trigger
Runtime builds a responder Chain for the current focus/source
Registry asks the chain for State or invocation claim
Target<C> executes and returns Response<C::Output>
Runtime applies response effects, history, invalidation, and observers
```

The target decides behavior. The responder chain decides where behavior is
available. Runtime coordinates the transaction.

## Boundary Rules

- Commands are contracts, not action ids.
- Targets execute commands; widgets do not.
- Responders describe capability scopes; they are not subject labels.
- Registry stores command metadata and typed/erased dispatch glue; it does not
  know focus or composition ancestry by itself.
- State describes current affordance and can hide a command from continued
  resolution.
- A public trait must not require crate-private values in its implementation.
  `Target<C>` therefore depends on public `command::State` and
  `response::Response` constructors.
- Shortcut resolution is platform-truthful data. Shortcut display is themed
  presentation.
- Erased command storage is private to the command/runtime boundary.
- If a public example needs crate-private command constructors, that is evidence
  for a deliberate application API pass, not a reason to leak internals
  incidentally.
