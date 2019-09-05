use crossbeam::Sender;
use cursive::event::{Event, EventResult, Key};
use cursive::view::View;
use cursive::views::Button;
use cursive::{Printer, Vec2};
use log::debug;
use std::fmt::Display;
use std::hash::Hash;

pub struct TabBar {
    children: Vec<Button>,
    // List of accumulated sizes of prev buttons
    sizes: Vec<Vec2>,
    idx: Option<usize>,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            sizes: Vec::new(),
            idx: None,
        }
    }

    pub fn add_button<K: Hash + Eq + Copy + Display + 'static>(&mut self, tx: Sender<K>, key: K) {
        let button = Button::new_raw(format!("│{}│", key), move |_| {
            debug!("send {}", key);
            match tx.send(key) {
                Ok(_) => {}
                Err(err) => {
                    debug!("button could not send key: {:?}", err);
                }
            }
        });
        self.children.push(button);
        self.idx = Some(self.children.len() - 1);
    }
}

impl View for TabBar {
    fn draw(&self, printer: &Printer) {
        // Horizontal split for children
        let mut count = 0;
        for child in &self.children {
            // There is no chainable api...
            let mut rel_sizes = self.sizes.clone();
            rel_sizes.truncate(count);
            let mut print = printer
                .offset(
                    rel_sizes
                        .iter()
                        .fold(Vec2::new(0, 0), |acc, x| acc.stack_horizontal(x))
                        .keep_x(),
                )
                .cropped(self.sizes[count]);
            // Set of focus to be focus even if the bar itself is not
            if let Some(focus) = self.idx {
                print.focused = focus == count;
            }
            count += 1;
            debug!("Printer for Button: is {:?}", print.size);
            debug!("With offset: {:?}", print.offset);
            child.draw(&print);
        }
    }

    fn layout(&mut self, vec: Vec2) {
        // Cannot borrow immutable later on
        let len = self.children.len();
        for child in &mut self.children {
            child.layout(Vec2::new(vec.x / len, vec.y));
        }
    }

    fn required_size(&mut self, cst: Vec2) -> Vec2 {
        self.sizes.clear();
        for child in &mut self.children {
            let size = child.required_size(cst);
            self.sizes.push(size);
        }
        self.sizes
            .clone()
            .iter()
            .fold(Vec2::new(0, 0), |acc, x| acc.stack_horizontal(x))
    }

    fn on_event(&mut self, evt: Event) -> EventResult {
        // TODO Mouse Events for all children

        if let Some(focus) = self.idx {
            debug!("Passing event {:?} to button {}.", evt.clone(), focus);
            let result = self.children[focus].on_event(evt.clone());
            match result {
                EventResult::Consumed(_) => {
                    debug!("Event has callback: {}", result.has_callback());
                    return result;
                }
                _ => {}
            }
        }

        match evt {
            Event::Key(Key::Left) => {
                if let Some(index) = self.idx {
                    if index > 0 {
                        self.idx = Some(index - 1);
                        EventResult::Consumed(None)
                    } else {
                        EventResult::Ignored
                    }
                } else {
                    EventResult::Ignored
                }
            }
            Event::Key(Key::Right) => {
                if let Some(index) = self.idx {
                    if index == (self.children.len() - 1) {
                        EventResult::Ignored
                    } else {
                        self.idx = Some(index + 1);
                        EventResult::Consumed(None)
                    }
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }
}
