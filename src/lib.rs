//! This crate provides a tabbing view for
//! [gyscos/cursive](https://github.com/gyscos/cursive) views. It is build to
//! be as simple as possible.
//!
//! The behaviour is oriented to be similar to  [`StackView`](https://docs.rs/cursive/0.13.0/cursive/views/struct.StackView.html) of cursive, but with the advantage of selectively displaying
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
//! #   tabs.add_tab("First", TextView::new("Our first view!"));
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
//!   tabs.add_tab("First", TextView::new("Our first view!"));
//!   siv.add_layer(Dialog::around(tabs));
//!   // When your done setting run cursive
//!   // siv.run();
//! }
//! ```
use crossbeam::{Receiver, Sender};
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
use bar::{Bar, TabBar};
pub use panel::{Align, TabPanel};

/// Main struct which manages views
pub struct TabView<K: Hash> {
    current_id: Option<K>,
    map: HashMap<K, Box<dyn View>>,
    key_order: Vec<K>,
    bar_rx: Option<Receiver<K>>,
    active_key_tx: Option<Sender<K>>,
    invalidated: bool,
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
    /// #  tabs.add_tab("First", TextView::new("Our first view!"));
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
            active_key_tx: None,
            invalidated: true,
        }
    }

    /// Returns the currently active tab Id.
    pub fn active_tab(&self) -> Option<K> {
        self.current_id
    }

    /// Set the currently active (visible) tab.
    /// If the tab id is not known, an error is returned and no action is performed.
    pub fn set_active_tab(&mut self, id: K) -> Result<(), ()> {
        if self.map.contains_key(&id) {
            if let Some(sender) = &self.active_key_tx {
                match sender.send(id) {
                    Ok(_) => {}
                    Err(e) => debug!(
                        "error occured while trying to send new active key to sender: {}",
                        e
                    ),
                }
            }
            self.current_id = Some(id);
            self.invalidated = true;
            Ok(())
        } else {
            Err(())
        }
    }

    /// Set the currently active (visible) tab.
    /// If the tab id is not known, an error is returned and no action is performed.
    ///
    /// This is the consumable variant.
    pub fn with_active_tab(mut self, id: K) -> Result<Self, ()> {
        self.set_active_tab(id)?;

        Ok(self)
    }

    /// Add a new tab to the tab view.
    /// The new tab will be set active and will be the visible tab for this tab view.
    pub fn add_tab<T: View>(&mut self, id: K, view: T) {
        self.map.insert(id, Box::new(view));
        self.current_id = Some(id);
        self.key_order.push(id);
    }

    /// Add a new tab to the tab view.
    /// The new tab will be set active and will be the visible tab for this tab view.
    ///
    /// This is the consumable variant.
    pub fn with_tab<T: View>(mut self, id: K, view: T) -> Self {
        self.add_tab(id, view);

        self
    }

    /// Add a new tab at a given position.
    /// The new tab will be set active and will be the visible tab for this tab view.
    ///
    /// This is designed to not fail, if the given position is greater than the number of current tabs, it simply will be appended.
    pub fn add_tab_at<T: View>(&mut self, id: K, view: T, pos: usize) {
        self.map.insert(id, Box::new(view));
        self.current_id = Some(id);
        if self.key_order.len() > pos {
            self.key_order.insert(pos, id)
        } else {
            self.key_order.push(id);
        }
    }

    /// Add a new tab at a given position.
    /// The new tab will be set active and will be the visible tab for this tab view.
    ///
    /// It is designed to be fail-safe, if the given position is greater than the number of current tabs, it simply will be appended.
    ///
    /// This is the consumable variant.
    pub fn with_tab_at<T: View>(mut self, id: K, view: T, pos: usize) -> Self {
        self.add_tab_at(id, view, pos);
        self
    }

    /// Swap the tabs position.
    /// If one of the given key cannot be found, then no operation is performed.
    pub fn swap_tabs(&mut self, fst: K, snd: K) {
        let mut fst_pos: Option<usize> = None;
        let mut snd_pos: Option<usize> = None;
        for (pos, key) in self.tab_order().into_iter().enumerate() {
            match key {
                val if val == fst => fst_pos = Some(pos),
                val if val == snd => snd_pos = Some(pos),
                _ => {}
            }
        }
        if fst_pos.is_some() && snd_pos.is_some() {
            self.key_order.swap(fst_pos.unwrap(), snd_pos.unwrap());
        }
    }

    /// Removes a tab with the given id from the `TabView`.
    /// If the removed tab is active at the moment, the `TabView` will unfocus it and
    /// the focus needs to be set manually afterwards, or a new view has to be inserted.
    pub fn remove_tab(&mut self, id: K) -> Result<(), ()> {
        if let Some(_) = self.map.remove(&id) {
            if let Some(key) = &self.current_id {
                if *key == id {
                    // Current id no longer valid
                    self.current_id = None;
                }
            }
            // remove_key experimental
            self.key_order = self
                .key_order
                .iter()
                .filter_map(|key| if id == *key { None } else { Some(*key) })
                .collect();
            self.invalidated = true;
            Ok(())
        } else {
            Err(())
        }
    }

    /// Returns the current order of keys in a vector.
    /// When you're implementing your own tab bar, be aware that this is the current
    /// tab bar and is only a copy of the original order, modification will not be
    /// transferred and future updates in the original not displayed.
    pub fn tab_order(&self) -> Vec<K> {
        self.key_order.clone()
    }

    // Returns the index of the key, length of the vector if the key is not included
    // This can be done with out sorting
    fn index_key(cur_key: &K, key_order: &Vec<K>) -> usize {
        for (idx, key) in key_order.iter().enumerate() {
            if *key == *cur_key {
                return idx;
            }
        }
        key_order.len()
    }

    /// Set the active tab to the next tab in order.
    pub fn next(&mut self) {
        if let Some(cur_key) = self.current_id {
            self.set_active_tab(
                self.key_order
                    [(Self::index_key(&cur_key, &self.key_order) + 1) % self.key_order.len()],
            )
            .expect("Key content changed during operation, this should not happen");
        }
    }

    /// Set the active tab to the previous tab in order.
    pub fn prev(&mut self) {
        if let Some(cur_key) = self.current_id {
            self.set_active_tab(
                self.key_order[(self.key_order.len() + Self::index_key(&cur_key, &self.key_order)
                    - 1)
                    % self.key_order.len()],
            )
            .expect("Key content changed during operation, this should not happen");
        }
    }

    /// Set the receiver for keys to be changed to
    pub fn set_bar_rx(&mut self, rx: Receiver<K>) {
        self.bar_rx = Some(rx);
    }

    /// Set the sender for the key switched to
    pub fn set_active_key_tx(&mut self, tx: Sender<K>) {
        self.active_key_tx = Some(tx);
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
        self.invalidated = false;
        if let Some(key) = &self.current_id {
            if let Some(view) = self.map.get_mut(&key) {
                view.layout(size);
            }
        }
    }

    fn required_size(&mut self, req: Vec2) -> Vec2 {
        if let Some(rx) = &self.bar_rx {
            if let Ok(evt) = rx.try_recv() {
                match self.set_active_tab(evt) {
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

    fn call_on_any<'a>(&mut self, slt: &Selector, mut cb: AnyCb<'a>) {
        for (_, view) in self.map.iter_mut() {
            view.call_on_any(slt, Box::new(|any| cb(any)));
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
        self.invalidated || {
            if let Some(key) = self.current_id {
                if let Some(view) = self.map.get(&key) {
                    view.needs_relayout()
                } else {
                    false
                }
            } else {
                false
            }
        }
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
        let mut tabs = TabView::<i32>::new().with_tab(0, DummyView);
        tabs.add_tab(1, DummyView);
    }

    #[test]
    fn switch() {
        let mut tabs = TabView::<i32>::new();
        tabs.add_tab(0, DummyView);
        tabs.add_tab(1, DummyView);
        assert_eq!(tabs.active_tab().expect("Id not correct"), 1);
        tabs.set_active_tab(0).expect("Id not taken");
        assert_eq!(tabs.active_tab().expect("Id not correct"), 0);
    }

    #[test]
    fn remove() {
        let mut tabs = TabView::<i32>::new();
        tabs.add_tab(0, DummyView);
        tabs.add_tab(1, DummyView);
        assert_eq!(tabs.remove_tab(1), Ok(()));
        assert!(tabs.active_tab().is_none());
    }
}
