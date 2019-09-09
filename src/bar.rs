use crossbeam::{Receiver, Sender};
use cursive::align::HAlign;
use cursive::event::{Event, EventResult, Key, MouseButton, MouseEvent};
use cursive::theme::{Effect, PaletteColor};
use cursive::view::{View, ViewWrapper};
use cursive::views::Button;
use cursive::{wrap_impl, Printer, Vec2};
use log::debug;
use std::fmt::Display;
use std::hash::Hash;

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
    h_align: HAlign,
    last_rendered_size: Vec2,
    // List of accumulated sizes of prev buttons
    sizes: Vec<Vec2>,
    idx: Option<usize>,
    rx: Receiver<K>,
}

impl<K: Hash + Eq + Copy + Display + 'static> TabBar<K> {
    pub fn new(rx: Receiver<K>) -> Self {
        Self {
            children: Vec::new(),
            sizes: Vec::new(),
            idx: None,
            h_align: HAlign::Left,
            bar_size: Vec2::zero(),
            last_rendered_size: Vec2::zero(),
            rx,
        }
    }

    pub fn h_align(mut self, align: HAlign) -> Self {
        self.h_align = align;
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
    }
}

impl<K: Hash + Eq + Copy + Display + 'static> View for TabBar<K> {
    fn draw(&self, printer: &Printer) {
        // First draw the complete horizontal line
        printer.print_hline((0, 0), printer.size.x, "─");
        printer.print((0, 0), "┌");
        printer.print((printer.size.x - 1, 0), "┐");
        let mut count = 0;
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
        for child in &self.children {
            // There is no chainable api...
            let mut rel_sizes = self.sizes.clone();
            rel_sizes.truncate(count);
            let mut print = inner_printer
                .offset(
                    rel_sizes
                        .iter()
                        .fold(Vec2::new(0, 0), |acc, x| acc.stack_horizontal(x))
                        .keep_x(),
                )
                // Spacing for first character
                .offset((count * 1, 0))
                .cropped({
                    if count == 0 || count == self.children.len() - 1 {
                        self.sizes[count].stack_horizontal(&Vec2::new(2, 1))
                    } else {
                        self.sizes[count].stack_horizontal(&Vec2::new(1, 1))
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
                print = print.focused(focus == count);
            }

            print.with_theme(&theme, |printer| {
                if count > 0 {
                    if child.active || self.children[count - 1].active {
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
                if count == self.children.len() - 1 {
                    if child.active {
                        printer
                            .offset((1, 0))
                            .print(self.sizes[count].keep_x(), "┠");
                    } else {
                        printer
                            .offset((1, 0))
                            .print(self.sizes[count].keep_x(), "├");
                    }
                }
            });
            count += 1;
        }
    }

    fn layout(&mut self, vec: Vec2) {
        for (child, size) in self.children.iter_mut().zip(self.sizes.iter()) {
            child.layout(*size);
        }
        self.last_rendered_size = vec;
    }

    fn required_size(&mut self, cst: Vec2) -> Vec2 {
        if let Ok(new_active) = self.rx.try_recv() {
            for child in &mut self.children {
                if new_active == child.key {
                    child.active = true;
                } else {
                    child.active = false;
                }
            }
        }
        self.sizes.clear();
        let mut start = Vec2::zero();
        for child in &mut self.children {
            let size = child.required_size(cst);
            start = start.stack_horizontal(&size.keep_x());
            child.pos = start;
            self.sizes.push(size);
        }
        // Total size of bar
        self.bar_size = start;
        // Return max width and maximum height of child
        Vec2::new(
            cst.x,
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
                let mut iter = self.children.iter().peekable();
                let mut count = 0;
                while let Some(child) = iter.next() {
                    if position.checked_sub(offset).is_some() {
                        if (child.pos
                            + Vec2::new(
                                {
                                    if count * 2 > 0 {
                                        count * 2
                                    } else {
                                        1
                                    }
                                },
                                0,
                            )
                            + Vec2::new(
                                self.h_align.get_offset(
                                    self.bar_size.x + self.children.len() + 1,
                                    self.last_rendered_size.x - 2,
                                ),
                                0,
                            ))
                        .fits(position - offset)
                        {
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
