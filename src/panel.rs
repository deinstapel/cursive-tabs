use crossbeam::{unbounded, Receiver, Sender};
use cursive::direction::{Absolute, Direction};
use cursive::event::{AnyCb, Event, EventResult, Key};
use cursive::view::{Selector, View};
use cursive::{Printer, Vec2};
use log::debug;
use std::fmt::Display;
use std::hash::Hash;

use crate::Bar;
use crate::TabBar;
use crate::TabView;

pub struct TabPanel<K: Hash + Eq + Display + Copy + 'static> {
    order: Vec<K>,
    bar: TabBar<K>,
    bar_size: Vec2,
    tx: Sender<K>,
    tabs: TabView<K>,
    active_rx: Receiver<K>,
    bar_focused: bool,
}

impl<K: Hash + Eq + Copy + Display + 'static> TabPanel<K> {
    pub fn new() -> Self {
        let mut tabs = TabView::new();
        let (tx, rx) = unbounded();
        let (active_tx, active_rx) = unbounded();
        tabs.bar_rx = Some(rx);
        tabs.active_key_tx = Some(active_tx);
        Self {
            order: Vec::new(),
            bar: TabBar::new(active_rx.clone()),
            bar_size: Vec2::new(1, 1),
            tabs,
            tx,
            active_rx,
            bar_focused: false,
        }
    }

    pub fn insert_view<T: View>(&mut self, id: K, view: T) -> K {
        self.tabs.insert_view(id, view)
    }

    pub fn with_view<T: View>(mut self, id: K, view: T) -> Self {
        self.tabs.insert_view(id, view);
        self
    }

    pub fn tab(&self) -> Option<K> {
        self.tabs.tab()
    }

    pub fn set_tab(&mut self, id: K) -> Result<(), ()> {
        self.tabs.set_tab(id)
    }

    pub fn remove_view(&mut self, id: K) -> Result<K, ()> {
        self.tabs.remove_view(id)
    }

    pub fn get_tab_order(&self) -> Vec<K> {
        self.tabs.get_tab_order()
    }

    pub fn next(&mut self) {
        self.tabs.next()
    }
}

impl<K: Hash + Eq + Copy + std::fmt::Display + 'static> View for TabPanel<K> {
    fn draw(&self, printer: &Printer) {
        // Side bars
        printer.print_vline((0, 0), printer.size.y, "│");
        printer.print_vline((printer.size.x - 1, 0), printer.size.y, "│");
        // Bottom line
        printer.print_hline((0, printer.size.y - 1), printer.size.x, "─");
        // Corners
        printer.print((0, printer.size.y - 1), "└");
        printer.print((printer.size.x - 1, printer.size.y - 1), "┘");
        let printer_bar = printer
            .cropped(Vec2::new(printer.size.x, self.bar_size.y))
            .focused(self.bar_focused);
        let printer_tab = printer
            .offset(Vec2::new(1, self.bar_size.y))
            // Inner area
            .cropped((printer.size.x - 2, printer.size.y - self.bar_size.y - 1))
            .focused(!self.bar_focused);
        self.bar.draw(&printer_bar);
        self.tabs.draw(&printer_tab);
    }

    fn layout(&mut self, vec: Vec2) {
        self.bar.layout(Vec2::new(vec.x, self.bar_size.y));
        self.tabs
            .layout(Vec2::new(vec.x - 2, vec.y - self.bar_size.y - 1));
    }

    fn needs_relayout(&self) -> bool {
        true
    }

    fn required_size(&mut self, cst: Vec2) -> Vec2 {
        if self.order != self.get_tab_order() {
            debug!("rebuild time!");
            self.bar = TabBar::new(self.active_rx.clone());
            for key in self.get_tab_order() {
                self.bar.add_button(self.tx.clone(), key);
            }
            self.order = self.get_tab_order();
        }
        let tab_size = self.tabs.required_size(cst);
        self.bar_size = self.bar.required_size(cst);
        self.bar_size.stack_vertical(&tab_size)
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
            let result = self.bar.on_event(evt.clone());
            match result {
                EventResult::Consumed(_) => result,
                EventResult::Ignored => match evt {
                    Event::Key(Key::Down) => match self.tabs.take_focus(Direction::up()) {
                        true => {
                            self.bar_focused = false;
                            EventResult::Consumed(None)
                        }
                        false => EventResult::Ignored,
                    },
                    _ => EventResult::Ignored,
                },
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
            Direction::Abs(Absolute::Down) => {
                if self.tabs.take_focus(d) {
                    self.bar_focused = false;
                } else {
                    self.bar_focused = true;
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
