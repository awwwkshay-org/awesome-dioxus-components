// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Implements the WAI-ARIA Authoring Practices Guide's "Trapping Focus" pattern for modal
// dialogs: on open, focus moves inside the container; Tab and Shift+Tab cycle only through
// the container's focusable descendants, wrapping from the last back to the first (and vice
// versa) instead of escaping to the rest of the page; on close, focus returns to whatever
// triggered the dialog. `FocusTrap` implements this with two mechanisms working together —
// a `keydown` listener that intercepts Tab/Shift+Tab directly, and a pair of invisible focus
// guard sentinels bracketing the container that redirect focus back inside if it ever lands
// outside by another route (for example a pointer click on a non-focusable area). `focusable()`
// also recognizes any element carrying an explicit `tabindex` attribute, not just a fixed set
// of natively-focusable tags, so it correctly walks roving-focus collections managed by this
// crate's `collection.rs` (listbox/menu items toggling `tabindex="0"`/`"-1"`). `FocusScope` is
// the same pattern's non-modal counterpart — APG's dismissible, non-trapping popups (menus,
// comboboxes) only need focus restored on close, not contained while open.

var focusable = function (element) {
  if (element.tabIndex < 0 || element.getAttribute("disabled")) return false;
  if (element.hasAttribute("tabindex")) return true;
  switch (element.tagName) {
    case "A":
      return !!element.href && element.rel !== "ignore";
    case "INPUT":
      return element.type !== "hidden";
    case "BUTTON":
    case "SELECT":
    case "TEXTAREA":
      return true;
    default:
      return false;
  }
};

function createFocusGuard() {
  var guard = document.createElement("span");
  guard.setAttribute("tabindex", "0");
  guard.setAttribute("aria-hidden", "true");
  guard.style.position = "fixed";
  guard.style.width = "1px";
  guard.style.height = "1px";
  guard.style.padding = "0";
  guard.style.margin = "-1px";
  guard.style.overflow = "hidden";
  guard.style.clip = "rect(0, 0, 0, 0)";
  guard.style.whiteSpace = "nowrap";
  guard.style.border = "0";
  return guard;
}

class FocusTrap {
  container;
  restoreFocusElement;
  nodeWalker;
  startGuard;
  endGuard;

  constructor(container) {
    this.container = container;
    this.restoreFocusElement = document.activeElement;
    this.nodeWalker = document.createTreeWalker(
      this.container,
      NodeFilter.SHOW_ELEMENT,
      {
        acceptNode: (node) => {
          if (node instanceof HTMLElement && focusable(node)) {
            return NodeFilter.FILTER_ACCEPT;
          }
          return NodeFilter.FILTER_SKIP;
        },
      },
    );

    this.startGuard = createFocusGuard();
    this.endGuard = createFocusGuard();
    this.startGuard.addEventListener("focus", () => this.focusNext());
    this.endGuard.addEventListener("focus", () => this.focusPrevious());
    this.container.parentNode?.insertBefore(this.startGuard, this.container);
    this.container.parentNode?.insertBefore(
      this.endGuard,
      this.container.nextSibling,
    );

    this.focusNext();

    this.container.addEventListener("keydown", (event) => {
      if (event.key === "Tab") {
        if (event.shiftKey) this.focusPrevious();
        else this.focusNext();
        event.preventDefault();
      }
    });
  }

  remove() {
    this.startGuard.remove();
    this.endGuard.remove();
    this.restoreFocusElement.focus();
  }

  focusChild(child) {
    child.focus();
  }

  focusNext() {
    const nextNode = this.nodeWalker.nextNode();
    if (nextNode) {
      this.focusChild(nextNode);
    } else {
      this.nodeWalker.currentNode = this.container;
      const nextNode2 = this.nodeWalker.nextNode();
      if (nextNode2) this.focusChild(nextNode2);
    }
  }

  focusPrevious() {
    const previousNode = this.nodeWalker.previousNode();
    if (previousNode) {
      this.focusChild(previousNode);
    } else {
      this.nodeWalker.currentNode = this.container;
      const lastNode = this.nodeWalker.lastChild();
      if (lastNode) this.focusChild(lastNode);
    }
  }
}

// A non-modal focus scope: unlike `FocusTrap`, it never traps Tab and never
// inserts focus guards, so the user can freely tab out of the container to
// the rest of the page. It only remembers what was focused before it was
// created, so `remove()` can restore focus on close -- the same courtesy a
// modal trap gives, without the containment.
class FocusScope {
  restoreFocusElement;

  constructor() {
    this.restoreFocusElement = document.activeElement;
  }

  remove() {
    this.restoreFocusElement.focus();
  }
}

window.createFocusTrap = (container) => {
  return new FocusTrap(container);
};

window.createFocusScope = () => {
  return new FocusScope();
};
