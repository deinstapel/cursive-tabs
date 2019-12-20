use crossbeam::{Receiver, Sender};
use cursive::event::{Event, EventResult, Key, MouseButton, MouseEvent};
use cursive::theme::{Effect, PaletteColor};
use cursive::view::{View, ViewWrapper};
use cursive::views::Button;
use cursive::{wrap_impl, Printer, Vec2};
use log::debug;
use std::fmt::Display;
use std::hash::Hash;

use crate::panel::Align;

pub trait Bar<K: Hash + Eq + Copy + Display + 'static> {
    fn add_button(&mut self, tx: Sender<K>, key: K);
}

// Quick Wrapper around Views to be able to set their positon
struct PositionWrap<T: View, K> {
    view: T,
    pub pos: Vec2,
    pub active: bool,
    pub key: K,
}

impl<T: View, K: 'static> ViewWrapper for PositionWrap<T, K> {
    wrap_impl!(self.view: T);
}

impl<T: View, K> PositionWrap<T, K> {
    pub fn new(view: T, key: K) -> Self {
        Self {
            view,
            pos: Vec2::zero(),
            active: false,
            key,
        }
    }
}

pub struct TabBar<K: Hash + Eq + Copy + Display + 'static> {
    children: Vec<PositionWrap<Button, K>>,
    bar_size: Vec2,
    h_align: Align,
    last_rendered_size: Vec2,
    // List of accumulated sizes of prev buttons
    sizes: Vec<Vec2>,
    idx: Option<usize>,
    rx: Receiver<K>,
    invalidated: bool,
}

impl<K: Hash + Eq + Copy + Display + 'static> TabBar<K> {
    pub fn new(rx: Receiver<K>) -> Self {
        Self {
            children: Vec::new(),
            sizes: Vec::new(),
            idx: None,
            h_align: Align::Start,
            bar_size: Vec2::zero(),
            last_rendered_size: Vec2::zero(),
            rx,
            invalidated: true,
        }
    }

    pub fn h_align(mut self, align: Align) -> Self {
        self.h_align = align;
        self.invalidated = true;
        self
    }
}

impl<K: Hash + Eq + Copy + Display + 'static> Bar<K> for TabBar<K> {
    fn add_button(&mut self, tx: Sender<K>, key: K) {
        let button = Button::new_raw(format!(" {} ", key), move |_| {
            debug!("send {}", key);
            match tx.send(key) {
                Ok(_) => {}
                Err(err) => {
                    debug!("button could not send key: {:?}", err);
                }
            }
        });
        self.children.push(PositionWrap::new(button, key));
        self.idx = Some(self.children.len() - 1);
        self.invalidated = true;
    }
}

impl<K: Hash + Eq + Copy + Display + 'static> View for TabBar<K> {
    fn draw(&self, printer: &Printer) {
        // First draw the complete horizontal line
        printer.print_hline((0, 0), printer.size.x, "─");
        printer.print((0, 0), "┌");
        printer.print((printer.size.x - 1, 0), "┐");
        // Spacing for padding & crop end
        let inner_printer = printer
            .offset((1, 0))
            // Alignment
            .offset((
                self.h_align.get_offset(
                    self.bar_size.x + self.children.len() + 1,
                    printer.size.x - 2,
                ),
                0,
            ))
            .cropped((printer.size.x - 2, printer.size.y));
        for (idx, child) in self.children.iter().enumerate() {
            // There is no chainable api...
            let mut rel_sizes = self.sizes.clone();
            rel_sizes.truncate(idx);
            let mut print = inner_printer
                .offset(
                    rel_sizes
                        .iter()
                        .fold(Vec2::new(0, 0), |acc, x| acc.stack_horizontal(x))
                        .keep_x(),
                )
                // Spacing for first character
                .offset((idx * 1, 0))
                .cropped({
                    if idx == 0 || idx == self.children.len() - 1 {
                        self.sizes[idx].stack_horizontal(&Vec2::new(2, 1))
                    } else {
                        self.sizes[idx].stack_horizontal(&Vec2::new(1, 1))
                    }
                });
            let mut theme = printer.theme.clone();

            if !child.active {
                let color = theme.palette[PaletteColor::TitleSecondary];
                theme.palette[PaletteColor::Primary] = color;
            } else {
                let color = theme.palette[PaletteColor::TitlePrimary];
                theme.palette[PaletteColor::Primary] = color;
            }

            if let Some(focus) = self.idx {
                print = print.focused(focus == idx);
            }

            print.with_theme(&theme, |printer| {
                if idx > 0 {
                    if child.active || self.children[idx - 1].active {
                        printer.print((0, 0), "┃")
                    } else {
                        printer.print((0, 0), "│");
                    }
                } else {
                    if child.active {
                        printer.print((0, 0), "┨")
                    } else {
                        printer.print((0, 0), "┤");
                    }
                }
                printer.with_effect(Effect::Bold, |printer| child.draw(&printer.offset((1, 0))));
                if idx == self.children.len() - 1 {
                    if child.active {
                        printer.offset((1, 0)).print(self.sizes[idx].keep_x(), "┠");
                    } else {
                        printer.offset((1, 0)).print(self.sizes[idx].keep_x(), "├");
                    }
                }
            });
        }
    }

    fn layout(&mut self, vec: Vec2) {
        self.invalidated = false;
        for (child, size) in self.children.iter_mut().zip(self.sizes.iter()) {
            child.layout(*size);
        }
        self.last_rendered_size = vec;
    }

    fn needs_relayout(&self) -> bool {
        self.invalidated
    }

    fn required_size(&mut self, cst: Vec2) -> Vec2 {
        if let Ok(new_active) = self.rx.try_recv() {
            self.invalidated = true;
            for child in &mut self.children {
                if new_active == child.key {
                    child.active = true;
                } else {
                    child.active = false;
                }
            }
        }
        self.sizes.clear();
        let sizes = &mut self.sizes;
        let total_size = self
            .children
            .iter_mut()
            .fold(Vec2::zero(), |mut acc, child| {
                let size = child.required_size(cst);
                acc = acc.stack_horizontal(&size.keep_x());
                child.pos = acc;
                sizes.push(size);
                acc
            });
        // Total size of bar
        self.bar_size = total_size;
        // Return max width and maximum height of child
        Vec2::new(
            cst.x,
            // Maximum height
            self.sizes.iter().fold(0, |mut val, x| {
                if val < x.y {
                    val = x.y;
                }
                val
            }),
        )
    }

    fn on_event(&mut self, evt: Event) -> EventResult {
        match evt.clone() {
            Event::Mouse {
                offset,
                position,
                event,
            } => {
                let mut iter = self.children.iter().peekable().enumerate();
                while let Some((idx, child)) = iter.next() {
                    if position.checked_sub(offset).is_some() {
                        if (child.pos
                            + Vec2::new(idx + 1, 0)
                            + Vec2::new(
                                self.h_align.get_offset(
                                    // Length of buttons and delimiting characters
                                    self.bar_size.x + self.children.len() + 1,
                                    self.last_rendered_size.x - 2,
                                ),
                                0,
                            ))
                        .fits(position - offset)
                        {
                            match event {
                                MouseEvent::Release(MouseButton::Left) => {
                                    self.invalidated = true;
                                    self.idx = Some(idx);
                                    return self.children[idx].on_event(Event::Key(Key::Enter));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        if let Some(focus) = self.idx {
            let pos = self.children[focus].pos;
            match self.children[focus].on_event(evt.relativized(pos)) {
                EventResult::Consumed(any) => {
                    self.invalidated = true;
                    return EventResult::Consumed(any);
                }
                _ => {}
            }
        }

        match evt {
            Event::Key(Key::Left) => {
                if let Some(index) = self.idx {
                    if index > 0 {
                        self.idx = Some(index - 1);
                        self.invalidated = true;
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
                        self.invalidated = true;
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
