# Key routing order

> Track 4 of [`RUNE_BORROW.md`](RUNE_BORROW.md). The order in which arniko offers a
> key to handlers, so a key that means different things at different times (Return,
> Escape, arrows) resolves the same way every time. Modelled on AppKit, which Rune
> follows. The focus *math* that steps 5 uses is in `arniko::focus`.

A key the OS delivers to a window is offered to the first of these that takes it:

1. **IME composition.** Return and Escape while a field composes (preedit) go to the
   composition and no further.
2. **Default button.** Return with no modifiers presses the window's (or open modal's)
   default button — unless the focused element takes Return (see step 5).
3. **Window `on_key`, nothing focused.** With no focused element, the window's key
   handler, told that focus is empty.
4. **Focused field input.** The focused field's `on_input_key`, which sees Return before
   any of the defaults below.
5. **Focus navigation.**
   - Escape dismisses the open modal.
   - Tab / Shift-Tab move to the next / previous Tab stop
     (`focus::tab_step` over `focus::tab_order`).
   - A focused slider's arrows and Page Up/Down step it; Home/End take it to its ends.
   - Arrows, Home, End and type-select move inside a `focus_group`
     (`focus::roving` / `focus::type_select`); menus, segments and tabs wrap, lists,
     trees and grids stop at the ends.
   - Space or Up/Down open a focused popup button (role `combobox`).
   - Space presses the focused control.
   - Return presses a focused row or link.
   - With Shift held, Tab and the navigation keys do none of this (typed capitals still
     type-select); they pass on.
6. **Focused text field editor.** The field's own editing keys.
7. **Window `on_key`, focus known.** The window's handler again, now told which element
   (or link) holds focus, for keys the focused control passed on.
8. **Selected-text copy keys.** Plain Copy / Select-All, for selected text. A focused
   text field keeps its own at step 6.
9. **Page scroll.** Page Up/Down and Home/End scroll the nearest scroll view around the
   focused element, else around the pointer, else the first in the tree, skipping any
   whose content fits.

**Return follows AppKit**: Return belongs to the default button, Space presses the
focused control.

| Focused | Return |
| --- | --- |
| Nothing, a button, checkbox, switch, radio, segment, popup button or slider | The default button, else the window `on_key` |
| A text field with `on_submit` | `on_submit` (Shift/Option-Return types a newline in a multi-line field) |
| A text field without `on_submit` | The default button |
| A row of a `menu`/`listbox`/`list`/`tree`/`grid` group, or a highlight row | Presses the row |
| A link in text | Opens the link |

A field's `on_input_key` sees Return before any of these.

## Status

The `arniko::focus` math (tab order, roving, type-select) is implemented and tested.
The routing *wheel* above is the target contract; wiring it into the event sink and the
engine's focus manager is follow-up work (the engine owns focus state today).
