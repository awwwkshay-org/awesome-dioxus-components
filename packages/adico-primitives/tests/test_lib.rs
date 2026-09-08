//! Black-box tests for crate-root hooks (`adico_primitives::use_escape_key`,
//! `use_controlled`, `use_optionally_controlled`), per this repo's
//! test-placement convention (see
//! `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline
//! in `src/lib.rs`.

use adico_primitives::use_escape_key;
use dioxus::html::{Code, HasKeyboardData, Location, Modifiers, ModifiersInteraction};
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct TestKeyboardData {
    key: Key,
}

impl ModifiersInteraction for TestKeyboardData {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}

impl HasKeyboardData for TestKeyboardData {
    fn key(&self) -> Key {
        self.key.clone()
    }
    fn code(&self) -> Code {
        Code::Escape
    }
    fn location(&self) -> Location {
        Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn escape_event() -> Event<KeyboardData> {
    Event::new(
        Rc::new(KeyboardData::new(TestKeyboardData { key: Key::Escape })),
        true,
    )
}

type EscapeHandler = Rc<RefCell<dyn FnMut(Event<KeyboardData>)>>;

/// Regression test for task 5.1/5.2 (`openspec/changes/deduplicate-primitives`):
/// isolates `use_escape_key`'s topmost-gating mechanism directly, bypassing
/// any specific consumer (`Menu`, `Tooltip`) and the full DOM-dispatch/
/// effect-flush machinery entirely -- see `test_menu.rs`'s
/// `an_escape_at_the_topmost_root_menu_still_closes_it` doc comment for why
/// a full-component version of this test was found to be confounded by
/// `Menu`'s own unrelated focus-sync effect. `Outer` and `Inner` are true
/// ancestor/descendant components (guaranteeing they share one
/// `adico_primitives::layer` stack -- see design.md's D12 for why sibling
/// components do *not* reliably share one), each registering via
/// `use_escape_key` and capturing its own returned handler into a slot the
/// test can invoke directly with a synthetic event, with no DOM dispatch or
/// re-render involved.
#[test]
fn only_the_topmost_of_two_nested_escape_registrants_reacts() {
    let outer_closed = Rc::new(Cell::new(false));
    let inner_closed = Rc::new(Cell::new(false));
    let outer_handler: Rc<RefCell<Option<EscapeHandler>>> = Rc::new(RefCell::new(None));
    let inner_handler: Rc<RefCell<Option<EscapeHandler>>> = Rc::new(RefCell::new(None));

    #[derive(Props, Clone)]
    struct InnerProps {
        closed: Rc<Cell<bool>>,
        handler_slot: Rc<RefCell<Option<EscapeHandler>>>,
    }
    impl PartialEq for InnerProps {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.closed, &other.closed)
        }
    }

    #[component]
    fn Inner(props: InnerProps) -> Element {
        let closed = props.closed.clone();
        let handler = use_escape_key(ReadSignal::new(Signal::new(true)), move || {
            closed.set(true);
        });
        *props.handler_slot.borrow_mut() = Some(Rc::new(RefCell::new(handler)));
        rsx! { "inner" }
    }

    #[derive(Props, Clone)]
    struct OuterProps {
        closed: Rc<Cell<bool>>,
        handler_slot: Rc<RefCell<Option<EscapeHandler>>>,
        inner_closed: Rc<Cell<bool>>,
        inner_handler_slot: Rc<RefCell<Option<EscapeHandler>>>,
    }
    impl PartialEq for OuterProps {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.closed, &other.closed)
        }
    }

    #[component]
    fn Outer(props: OuterProps) -> Element {
        let closed = props.closed.clone();
        let handler = use_escape_key(ReadSignal::new(Signal::new(true)), move || {
            closed.set(true);
        });
        *props.handler_slot.borrow_mut() = Some(Rc::new(RefCell::new(handler)));
        rsx! {
            Inner {
                closed: props.inner_closed.clone(),
                handler_slot: props.inner_handler_slot.clone(),
            }
        }
    }

    let mut dom = VirtualDom::new_with_props(
        Outer,
        OuterProps {
            closed: outer_closed.clone(),
            handler_slot: outer_handler.clone(),
            inner_closed: inner_closed.clone(),
            inner_handler_slot: inner_handler.clone(),
        },
    );
    dom.rebuild_in_place();

    // `Inner` is a true descendant of `Outer`, mounted after it, so it is
    // the topmost registrant on their shared layer stack.
    (outer_handler.borrow().as_ref().unwrap().borrow_mut())(escape_event());
    assert!(
        !outer_closed.get(),
        "the non-topmost (outer) registrant must not react to Escape"
    );
    assert!(!inner_closed.get(), "inner was never sent an event yet");

    (inner_handler.borrow().as_ref().unwrap().borrow_mut())(escape_event());
    assert!(
        inner_closed.get(),
        "the topmost (inner) registrant must react to Escape"
    );
    assert!(
        !outer_closed.get(),
        "the outer registrant must still be untouched by inner's own Escape"
    );
}
