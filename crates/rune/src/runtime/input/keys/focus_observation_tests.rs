use crate::{Cx, El, Env, Key, KeyEvent, Mods, Runtime, ScrollSpec, TextEdit, TextStyle, col, rich, text, text_input};

fn agrees<S: 'static>(rt: &Runtime<S>) {
    let expected = rt.elements().into_iter().find(|e| e.focused && !e.exiting).and_then(|e| e.key);
    assert_eq!(rt.presented_focus_key(), expected.as_deref());
}

struct State { edit: TextEdit, show: bool }

fn view(s: &State, _: &mut Cx<State>) -> El {
    col().size(300.0, 240.0).gap(10.0)
        .children(s.show.then(|| text_input(&s.edit, TextStyle::new(14.0, 400.0))
            .key("editor").w(200.0).h(40.0).on_edit(|s: &mut State| &mut s.edit)))
        .child(text("Action").key("action").h(40.0).focusable(true))
        .child(text("Unkeyed").h(40.0).focusable(true))
}

#[test]
fn presented_focus_follows_pointer_tab_blur_and_removal() {
    let mut rt = Runtime::new(State { edit: TextEdit::new("draft"), show: true }, view, (300.0, 240.0), Env::new(true, 1.0));
    rt.frame(0.0);
    agrees(&rt);
    let (x, y) = rt.find("editor").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert_eq!(rt.presented_focus_key(), Some("editor"));
    for expected in [Some("action"), None, Some("editor")] {
        rt.key(KeyEvent { key: Key::Tab, mods: Mods::default() });
        rt.frame(0.0);
        agrees(&rt);
        assert_eq!(rt.presented_focus_key(), expected);
    }
    rt.update(|s, _| s.show = false);
    for _ in 0..30 {
        rt.frame(1.0 / 60.0);
        agrees(&rt);
        assert_ne!(rt.presented_focus_key(), Some("editor"));
    }
    rt.update(|_, tx| tx.blur());
    rt.frame(0.0);
    agrees(&rt);
    assert_eq!(rt.presented_focus_key(), None);
}

#[test]
fn presented_focus_reports_link_identity_and_clears_removed_links() {
    fn view(show: &bool, _: &mut Cx<bool>) -> El {
        col().size(300.0, 120.0).children(show.then(|| rich("See docs", Vec::new()).key("paragraph")
            .link(crate::Link::new(4..8, "docs")).on_link(|_: &mut bool, _, _| {})))
    }
    let mut rt = Runtime::new(true, view, (300.0, 120.0), Env::new(true, 1.0));
    rt.frame(0.0);
    rt.key(KeyEvent { key: Key::Tab, mods: Mods::default() });
    rt.frame(0.0);
    agrees(&rt);
    assert_eq!(rt.presented_focus_key(), Some("docs"));
    rt.update(|s, _| *s = false);
    for _ in 0..30 { rt.frame(1.0 / 60.0); agrees(&rt); }
    assert_eq!(rt.presented_focus_key(), None);
}

#[cfg(feature = "accessibility")]
#[test]
fn presented_focus_observes_accessibility_navigation() {
    let mut rt = Runtime::new(State { edit: TextEdit::new("draft"), show: true }, view, (300.0, 240.0), Env::new(true, 1.0));
    rt.frame(0.0);
    assert!(rt.accessibility_action("action", crate::accesskit::Action::Focus, None));
    rt.frame(0.0);
    agrees(&rt);
    assert_eq!(rt.presented_focus_key(), Some("action"));
}

#[test]
fn presented_focus_omits_unmounted_virtual_editors_and_recovers_on_reveal() {
    fn view(s: &Vec<TextEdit>, _: &mut Cx<Vec<TextEdit>>) -> El {
        col().w_full().h_full().scroll(ScrollSpec { stick_bottom: false }).child(
            col().w_full().children(s.iter().enumerate().map(|(i, edit)| {
                text_input(edit, TextStyle::new(14.0, 400.0)).key(format!("field-{i}")).w_full().h(40.0)
                    .on_edit(move |s: &mut Vec<TextEdit>| &mut s[i])
            })).growth(crate::GrowthSpec::default()).virtualize(40.0)
        )
    }
    let mut fields: Vec<_> = (0..200).map(|i| TextEdit::new(format!("Field {i}"))).collect();
    fields[180].focused = true;
    let mut rt = Runtime::new(fields, view, (300.0, 200.0), Env::new(true, 1.0));
    rt.frame(0.0);
    assert!(rt.has_focus());
    agrees(&rt);
    assert_eq!(rt.presented_focus_key(), None);
    rt.update(|_, tx| tx.scroll_into_view("field-180"));
    for _ in 0..30 { rt.frame(1.0 / 60.0); agrees(&rt); }
    assert_eq!(rt.presented_focus_key(), Some("field-180"));
}
