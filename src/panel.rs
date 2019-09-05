use crossbeam::{unbounded, Sender};
use cursive::direction::Direction;
use cursive::event::{AnyCb, Event, EventResult, Key};
use cursive::view::{Selector, View};
use cursive::views::{Panel};
use cursive::{Printer, Vec2};
use log::debug;
use std::hash::Hash;

use crate::TabBar;

use crate::TabView;

pub struct TabPanel<K: Hash + Eq + Copy + 'static> {
    order: Vec<K>,
    bar: TabBar,
    bar_size: Vec2,
    tx: Sender<K>,
    tabs: Panel<TabView<K>>,
    bar_focused: bool,
}

impl<K: Hash + Eq + Copy + 'static> TabPanel<K> {
    pub fn new() -> Self {
        let mut tabs = Panel::new(TabView::new());
        let (tx, rx) = unbounded();
        tabs.get_inner_mut().bar_rx = Some(rx);
        Self {
            order: Vec::new(),
            bar: TabBar::new(),
            bar_size: Vec2::new(1, 1),
            tabs,
            tx,
            bar_focused: false,
        }
    }

    pub fn insert_view<T: View>(&mut self, id: K, view: T) -> K {
        self.tabs.get_inner_mut().insert_view(id, view)
    }

    pub fn with_view<T: View>(mut self, id: K, view: T) -> Self {
        self.tabs.get_inner_mut().insert_view(id, view);
        self
    }

    pub fn tab(&self) -> Option<K> {
        self.tabs.get_inner().tab()
    }

    pub fn set_tab(&mut self, id: K) -> Result<(), ()> {
        self.tabs.get_inner_mut().set_tab(id)
    }

    pub fn remove_view(&mut self, id: K) -> Result<K, ()> {
        self.tabs.get_inner_mut().remove_view(id)
    }

    pub fn get_tab_order(&self) -> Vec<K> {
        self.tabs.get_inner().get_tab_order()
    }
}

impl<K: Hash + Eq + Copy + std::fmt::Display + 'static> View for TabPanel<K> {
    fn draw(&self, printer: &Printer) {
        let printer_bar = printer
            .cropped(Vec2::new(printer.size.x, self.bar_size.y))
            .focused(self.bar_focused);
        let printer_tab = printer
            .offset(Vec2::new(0, self.bar_size.y))
            .focused(!self.bar_focused);
        self.bar.draw(&printer_bar);
        self.tabs.draw(&printer_tab);
    }

    fn layout(&mut self, vec: Vec2) {
        self.bar.layout(Vec2::new(vec.x, self.bar_size.y));
        self.tabs.layout(Vec2::new(vec.x, vec.y - self.bar_size.y));
    }

    fn needs_relayout(&self) -> bool {
        true
    }

    fn required_size(&mut self, cst: Vec2) -> Vec2 {
        if self.order != self.get_tab_order() {
            debug!("rebuild time!");
            self.bar = TabBar::new();
            for key in self.get_tab_order() {
                self.bar.add_button(self.tx.clone(), key);
            }
            self.order = self.get_tab_order();
        }
        self.bar_size = self.bar.required_size(cst);
        self.bar_size.stack_vertical(&self.tabs.required_size(cst))
    }

    fn on_event(&mut self, evt: Event) -> EventResult {
        match evt.clone() {
            Event::Mouse {
                offset,
                position,
                event: _,
            } => {
                debug!(
                    "mouse event: offset: {:?} , position: {:?}",
                    offset, position
                );
                if position > offset {
                    if (position - offset).fits_in(self.bar_size) {
                        self.bar_focused = true;
                    } else {
                        self.bar_focused = false;
                    }
                }
            }
            _ => {}
        }

        if self.bar_focused {
            debug!("event to bar please respond!: {:?}", evt.clone());
            let result = self.bar.on_event(evt.clone());
            match result {
                EventResult::Consumed(_) => result,
                EventResult::Ignored => {
                    debug!("but it rejected!");
                    match evt {
                        Event::Key(Key::Down) => {
                            self.bar_focused = false;
                            EventResult::Consumed(None)
                        }
                        _ => EventResult::Ignored,
                    }
                }
            }
        } else {
            match self.tabs.on_event(evt.relativized((0, self.bar_size.y))) {
                EventResult::Consumed(_) => EventResult::Consumed(None),
                EventResult::Ignored => match evt {
                    Event::Key(Key::Up) => {
                        self.bar_focused = true;
                        if self.tabs.take_focus(Direction::up()) {
                            EventResult::Consumed(None)
                        } else {
                            EventResult::Ignored
                        }
                    }
                    _ => EventResult::Ignored,
                },
            }
        }
    }

    fn take_focus(&mut self, d: Direction) -> bool {
        match d {
            _ if d == Direction::down() => {
                if self.tabs.take_focus(d) {
                    self.bar_focused = false;
                }
            }
            _ => {
                self.bar_focused = true;
            }
        }
        true
    }

    fn focus_view(&mut self, slt: &Selector) -> Result<(), ()> {
        self.tabs.focus_view(slt)
    }

    fn call_on_any<'a>(&mut self, slt: &Selector, mut cb: AnyCb<'a>) {
        self.bar.call_on_any(slt, Box::new(|any| cb(any)))
    }
}
