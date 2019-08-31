use cursive::direction::Direction;
use cursive::event::{AnyCb, Event, EventResult};
use cursive::view::{Selector, View};
use cursive::{Printer, Rect, Vec2};
use std::collections::HashMap;
use std::hash::Hash;

pub struct TabView<K: Hash> {
    current_id: Option<K>,
    map: HashMap<K, Box<dyn View>>,
}

impl<K: Hash + Eq + Copy + 'static> TabView<K> {
    pub fn new() -> Self {
        Self {
            current_id: None,
            map: HashMap::new(),
        }
    }

    pub fn set_tab(&mut self, id: K) -> Result<(), ()> {
        if self.map.contains_key(&id) {
            self.current_id = Some(id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn with_view<T: View>(mut self, id: K, view: T) -> Self {
        self.map.insert(id, Box::new(view));
        self.current_id = Some(id);
        self
    }

    pub fn insert_view<T: View>(&mut self, id: K, view: T) -> K {
        self.map.insert(id, Box::new(view));
        self.current_id = Some(id);
        id
    }

    pub fn tab(&self) -> Option<K> {
        self.current_id
    }

    pub fn remove_view(&mut self, id: K) -> Result<K, ()> {
        if let Some(_) = self.map.remove(&id) {
            if let Some(key) = &self.current_id {
                if *key == id {
                    // Current id no longer valid
                    self.current_id = None;
                }
            }
            Ok(id)
        } else {
            Err(())
        }
    }
}

impl<K: Hash + Eq + 'static> View for TabView<K> {
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
        if let Some(key) = &self.current_id {
            if let Some(view) = self.map.get(&key) {
                view.needs_relayout()
            } else {
                false
            }
        } else {
            false
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

    //fn with_view<F, R>(&self, f: F) -> Option<R>
    //where
    //    F: FnOnce(&Self::V) -> R
    //{
    //    if let Some(key) = self.current_id {
    //        Some(f(self.map.get(&key)))
    //    } else {
    //        None
    //    }
    //}

    //fn with_view_mut<F, R>(&mut self, f: F) -> Option<R>
    //where
    //    F: FnOnce(&mut Self::V) -> R
    //{
    //    if let Some(key) = self.current_id {
    //        Some(f(self.map.get_mut(&key)))
    //    } else {
    //        None
    //    }
    //}
}

#[cfg(test)]
mod test{
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
