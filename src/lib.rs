//! This crate provides a tabbing view for [gyscos/cursive](https://github.com/gyscos/cursive) views. It is build to be as simple as possible.
//!
//! The behaviour is oriented at the [`StackView`](https://docs.rs/cursive/0.13.0/cursive/views/struct.StackView.html) of cursive, but with the advantage of selectively displaying
//! views without needing to delete foremost one.
//!
//! # Example
//! All you need to do to create a new `TabView` is:
//! ```
//! # use cursive::{Cursive, views::{TextView, Dialog}};
//! # use cursive_tabs::TabView;
//! # fn main() {
//! #   let mut siv = Cursive::default();
//! let mut tabs = TabView::new();
//! #   // That is all what is needed to display an empty TabView, but of course
//! #   // you can add your own tabs now and switch them around as you want!
//! #   tabs.insert_view("First", TextView::new("Our first view!"));
//! #   siv.add_layer(Dialog::around(tabs));
//! #   // When your done setting run cursive
//! #   // siv.run();
//! # }
//! ```
//! You can then use the provided methods to modify the content of the `TabView`
//! Consuming and non-consuming are both provided.
//!
//! # Full Example
//! ```
//! use cursive::{Cursive, views::{TextView, Dialog}};
//! use cursive_tabs::TabView;
//! fn main() {
//!   let mut siv = Cursive::default();
//!   let mut tabs = TabView::new();
//!   // That is all what is needed to display an empty TabView, but of course
//!   // you can add your own tabs now and switch them around as you want!
//!   tabs.insert_view("First", TextView::new("Our first view!"));
//!   siv.add_layer(Dialog::around(tabs));
//!   // When your done setting run cursive
//!   // siv.run();
//! }
//! ```
use crossbeam::Receiver;
use cursive::direction::Direction;
use cursive::event::{AnyCb, Event, EventResult};
use cursive::view::{Selector, View};
use cursive::{Printer, Rect, Vec2};
use log::debug;
use std::collections::HashMap;
use std::hash::Hash;

mod bar;
mod panel;

// Reexports
use bar::{TabBar, Bar};
pub use panel::TabPanel;

/// Main struct which manages views
pub struct TabView<K: Hash> {
    current_id: Option<K>,
    map: HashMap<K, Box<dyn View>>,
    key_order: Vec<K>,
    bar_rx: Option<Receiver<K>>,
}

impl<K: Hash + Eq + Copy + 'static> TabView<K> {
    /// Returns a new TabView
    /// # Example
    /// ```
    /// # use cursive::{Cursive, views::{TextView, Dialog}};
    /// # use cursive_tabs::TabView;
    /// # fn main() {
    /// #  let mut siv = Cursive::default();
    /// let mut tabs = TabView::new();
    /// #  // That is all what is needed to display an empty TabView, but of course
    /// #  // you can add your own tabs now and switch them around as you want!
    /// #  tabs.insert_view("First", TextView::new("Our first view!"));
    /// #  siv.add_layer(Dialog::around(tabs));
    /// #  // When your done setting run cursive
    /// #  // siv.run();
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            current_id: None,
            map: HashMap::new(),
            key_order: Vec::new(),
            bar_rx: None,
        }
    }

    /// Non-consuming setting of the to be active tab.
    /// If the tab id is not known, an error is returned and no action is performed.
    pub fn set_tab(&mut self, id: K) -> Result<(), ()> {
        if self.map.contains_key(&id) {
            self.current_id = Some(id);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Consuming insertion of a new view.
    /// Active tab will be switched to the newly inserted one.
    pub fn with_view<T: View>(mut self, id: K, view: T) -> Self {
        self.map.insert(id, Box::new(view));
        self.current_id = Some(id);
        self.key_order.push(id);
        self
    }

    /// Non-consuming insertion of a new view.
    /// Active tab will be switched to the newly inserted one.
    pub fn insert_view<T: View>(&mut self, id: K, view: T) -> K {
        self.map.insert(id, Box::new(view));
        self.current_id = Some(id);
        self.key_order.push(id);
        id
    }

    /// Returns the currently active tab Id.
    pub fn tab(&self) -> Option<K> {
        self.current_id
    }

    /// Removes the given id from the `TabView`.
    /// If the removed view is active at the moment, the `TabView` will unfocus it and the focus needs to be set manually afterwards, or a new view has to be inserted.
    pub fn remove_view(&mut self, id: K) -> Result<K, ()> {
        if let Some(_) = self.map.remove(&id) {
            if let Some(key) = &self.current_id {
                if *key == id {
                    // Current id no longer valid
                    self.current_id = None;
                }
            }
            self.key_order = self
                .key_order
                .iter()
                .filter_map(|key| if id == *key { None } else { Some(*key) })
                .collect();
            Ok(id)
        } else {
            Err(())
        }
    }

    /// Returns the current order of keys in a vector.
    /// When you're implementing your own tab bar, be aware that this is the current tab bar and is only a copy of the original order, modification will not be transferred and future updates in the original not displayed.
    pub fn get_tab_order(&self) -> Vec<K> {
        self.key_order.clone()
    }
}

impl<K: Hash + Eq + Copy + 'static> View for TabView<K> {
    fn draw(&self, printer: &Printer) {
        if let Some(key) = &self.current_id {
            if let Some(view) = self.map.get(&key) {
                view.draw(printer);
            }
        }
    }

    fn layout(&mut self, size: Vec2) {
        if let Some(key) = &self.current_id {
            if let Some(view) = self.map.get_mut(&key) {
                view.layout(size);
            }
        }
    }

    fn required_size(&mut self, req: Vec2) -> Vec2 {
        if let Some(rx) = &self.bar_rx {
            if let Ok(evt) = rx.try_recv() {
                match self.set_tab(evt) {
                    Ok(_) => {}
                    Err(err) => debug!("could not accept tab bar event: {:?}", err),
                }
            }
        }
        if let Some(key) = &self.current_id {
            if let Some(view) = self.map.get_mut(&key) {
                view.required_size(req)
            } else {
                (1, 1).into()
            }
        } else {
            (1, 1).into()
        }
    }

    fn on_event(&mut self, evt: Event) -> EventResult {
        if let Some(key) = &self.current_id {
            if let Some(view) = self.map.get_mut(&key) {
                view.on_event(evt)
            } else {
                EventResult::Ignored
            }
        } else {
            EventResult::Ignored
        }
    }

    fn take_focus(&mut self, src: Direction) -> bool {
        if let Some(key) = &self.current_id {
            if let Some(view) = self.map.get_mut(&key) {
                view.take_focus(src)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn call_on_any<'a>(&mut self, slt: &Selector, cb: AnyCb<'a>) {
        if let Some(key) = &self.current_id {
            if let Some(view) = self.map.get_mut(&key) {
                view.call_on_any(slt, cb);
            }
        }
    }

    fn focus_view(&mut self, slt: &Selector) -> Result<(), ()> {
        if let Some(key) = &self.current_id {
            if let Some(view) = self.map.get_mut(&key) {
                view.focus_view(slt)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    fn needs_relayout(&self) -> bool {
        true
    }

    fn important_area(&self, size: Vec2) -> Rect {
        if let Some(key) = &self.current_id {
            if let Some(view) = self.map.get(&key) {
                view.important_area(size)
            } else {
                Rect::from((1, 1))
            }
        } else {
            Rect::from((1, 1))
        }
    }
}

#[cfg(test)]
mod test {
    use super::TabView;
    use cursive::views::DummyView;

    #[test]
    fn smoke() {
        let _ = TabView::<i32>::new();
    }

    #[test]
    fn insert() {
        let mut tabs = TabView::<i32>::new().with_view(0, DummyView);
        tabs.insert_view(1, DummyView);
    }

    #[test]
    fn switch() {
        let mut tabs = TabView::<i32>::new();
        tabs.insert_view(0, DummyView);
        tabs.insert_view(1, DummyView);
        assert_eq!(tabs.tab().expect("Id not correct"), 1);
        tabs.set_tab(0).expect("Id not taken");
        assert_eq!(tabs.tab().expect("Id not correct"), 0);
    }

    #[test]
    fn remove() {
        let mut tabs = TabView::<i32>::new();
        tabs.insert_view(0, DummyView);
        tabs.insert_view(1, DummyView);
        assert_eq!(tabs.remove_view(1).unwrap(), 1);
        assert!(tabs.tab().is_none());
    }
}
