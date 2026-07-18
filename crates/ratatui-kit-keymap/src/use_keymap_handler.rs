//! The [`UseKeymapHandler`] hook: dispatch key events as keymap actions.

use std::hash::Hash;
use std::sync::Arc;

use ratatui_kit::{
    EventPriority, EventResult, EventScope, Hooks, UseEventHandler,
    crossterm::event::{Event, KeyEvent, KeyEventKind},
};

use crate::Keymap;

mod private {
    pub trait Sealed {}
    impl Sealed for ratatui_kit::Hooks<'_, '_> {}
}

/// Hook extension: register an event handler that resolves key presses
/// through a [`Keymap`] and dispatches by action.
pub trait UseKeymapHandler: private::Sealed {
    /// Register a key handler driven by `keymap`.
    ///
    /// Only `KeyEventKind::Press` key events are considered. When the pressed
    /// combination is bound, `f(action, key_event)` runs and its
    /// [`EventResult`] is returned; everything else is `Ignored` so the event
    /// keeps flowing to other handlers (app shell keys etc.).
    ///
    /// Handlers re-register every frame, so this is called once per frame.
    /// Store the merged keymap in an `Arc<Keymap<A>>` and pass a clone — a
    /// refcount bump, not a deep copy. (Passing an owned `Keymap` also works;
    /// it is wrapped in a fresh `Arc`.) How the keymap is distributed
    /// (context, `Atom`, props) is the host's choice.
    fn use_keymap_handler<A, K, F>(
        &mut self,
        scope: EventScope,
        priority: EventPriority,
        keymap: K,
        f: F,
    ) where
        A: Copy + Eq + Hash + 'static,
        K: Into<Arc<Keymap<A>>>,
        F: FnMut(A, KeyEvent) -> EventResult + 'static;
}

impl UseKeymapHandler for Hooks<'_, '_> {
    fn use_keymap_handler<A, K, F>(
        &mut self,
        scope: EventScope,
        priority: EventPriority,
        keymap: K,
        mut f: F,
    ) where
        A: Copy + Eq + Hash + 'static,
        K: Into<Arc<Keymap<A>>>,
        F: FnMut(A, KeyEvent) -> EventResult + 'static,
    {
        let keymap: Arc<Keymap<A>> = keymap.into();
        self.use_event_handler(scope, priority, move |event| {
            let Event::Key(key) = event else {
                return EventResult::Ignored;
            };
            if key.kind != KeyEventKind::Press {
                return EventResult::Ignored;
            }
            match keymap.action_for(key) {
                Some(action) => f(action, key),
                None => EventResult::Ignored,
            }
        });
    }
}
