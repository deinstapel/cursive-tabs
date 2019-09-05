use crossbeam::Sender;
use cursive::event::{Event, EventResult, Key, MouseButton, MouseEvent};
use cursive::view::{View, ViewWrapper};
use cursive::views::Button;
use cursive::{wrap_impl, Printer, Vec2};
use log::debug;
use std::fmt::Display;
use std::hash::Hash;

pub struct TabBar {
    children: Vec<PositionWrap<Button>>,
    // List of accumulated sizes of prev buttons
    sizes: Vec<Vec2>,
    idx: Option<usize>,
}

pub trait Bar {
    fn add_button<K: Hash + Eq + Copy + Display + 'static>(&mut self, tx: Sender<K>, key: K);
}

// Quick Wrapper around Views to be able to set their positon
struct PositionWrap<T: View> {
    view: T,
    pub pos: Vec2,
}

impl<T: View> ViewWrapper for PositionWrap<T> {
    wrap_impl!(self.view: T);
}

impl<T: View> PositionWrap<T> {
    pub fn new(view: T) -> Self {
        Self {
            view,
            pos: Vec2::zero(),
        }
    }
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            sizes: Vec::new(),
            idx: None,
        }
    }
}

impl Bar for TabBar {
    fn add_button<K: Hash + Eq + Copy + Display + 'static>(&mut self, tx: Sender<K>, key: K) {
        let button = Button::new_raw(format!("│{}│", key), move |_| {
            debug!("send {}", key);
            match tx.send(key) {
                Ok(_) => {}
                Err(err) => {
                    debug!("button could not send key: {:?}", err);
                }
            }
        });
        self.children.push(PositionWrap::new(button));
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
                print.enabled = focus == count;
            }
            count += 1;
            debug!("Printer for Button: is {:?}", print.size);
            debug!("With offset: {:?}", print.offset);
            child.draw(&print);
        }
    }

    fn layout(&mut self, _vec: Vec2) {
        for (child, size) in self.children.iter_mut().zip(self.sizes.iter()) {
            child.layout(*size);
        }
    }

    fn required_size(&mut self, cst: Vec2) -> Vec2 {
        self.sizes.clear();
        let mut start = Vec2::zero();
        for child in &mut self.children {
            let size = child.required_size(cst);
            start = start.stack_horizontal(&size.keep_x());
            child.pos = start;
            self.sizes.push(size);
        }
        self.sizes
            .clone()
            .iter()
            .fold(Vec2::new(0, 0), |acc, x| acc.stack_horizontal(x))
    }

    fn on_event(&mut self, evt: Event) -> EventResult {
        // TODO Mouse Events for all children
        match evt.clone() {
            Event::Mouse {
                offset,
                position,
                event,
            } => {
                let mut iter = self.children.iter().peekable();
                let mut count = 0;
                while let Some(child) = iter.next() {
                    if position.checked_sub(offset).is_some() {
                        if child.pos.fits(position - offset) {
                            debug!("hit");
                            match event {
                                MouseEvent::Release(MouseButton::Left) => {
                                    self.idx = Some(count);
                                    return self.children[count].on_event(Event::Key(Key::Enter));
                                }
                                _ => {}
                            }
                        }
                        count += 1;
                    }
                }
            }
            _ => {}
        }

        if let Some(focus) = self.idx {
            debug!("Passing event {:?} to button {}.", evt.clone(), focus);
            let pos = self.children[focus].pos;
            debug!("evt location: {:?}", evt.relativized(pos));
            let result = self.children[focus].on_event(evt.relativized(pos));
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
