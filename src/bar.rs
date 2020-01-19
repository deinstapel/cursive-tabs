use crossbeam::{Receiver, Sender};
use cursive::event::{Event, EventResult, Key, MouseButton, MouseEvent};
use cursive::theme::{Effect, PaletteColor};
use cursive::view::{View, ViewWrapper};
use cursive::views::Button;
use cursive::{wrap_impl, Printer, Vec2};
use log::debug;
use std::fmt::Display;
use std::hash::Hash;

use crate::panel::{Align, Placement};

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
    align: Align,
    last_rendered_size: Vec2,
    // List of accumulated sizes of prev buttons
    sizes: Vec<Vec2>,
    placement: Placement,
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
            align: Align::Start,
            placement: Placement::HorizontalTop,
            bar_size: Vec2::zero(),
            last_rendered_size: Vec2::zero(),
            rx,
            invalidated: true,
        }
    }

    pub fn with_alignment(mut self, align: Align) -> Self {
        self.align = align;
        self.invalidated = true;
        self
    }

    pub fn with_placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self.invalidated = true;
        self
    }

    fn decrement_idx(&mut self) -> EventResult {
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

    fn increment_idx(&mut self) -> EventResult {
        if let Some(index) = self.idx {
            if (index + 1) < self.children.len() {
                self.idx = Some(index + 1);
                self.invalidated = true;
                EventResult::Consumed(None)
            } else {
                EventResult::Ignored
            }
        } else {
            EventResult::Ignored
        }
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
        match self.placement {
            Placement::HorizontalBottom | Placement::HorizontalTop => {
                // First draw the complete horizontal line
                printer.print_hline((0, 0), printer.size.x, "─");
                // Spacing for padding & crop end
                let inner_printer = printer
                    // Alignment
                    .offset((
                        self.align
                            .get_offset(self.bar_size.x + self.children.len() + 1, printer.size.x),
                        0,
                    ));
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
                        .offset((idx, 0))
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
                        } else if child.active {
                            printer.print((0, 0), "┨")
                        } else {
                            printer.print((0, 0), "┤");
                        }
                        printer.with_effect(Effect::Bold, |printer| {
                            child.draw(&printer.offset((1, 0)))
                        });
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
            Placement::VerticalLeft | Placement::VerticalRight => {
                // First draw the complete vertical line
                let horizontal_offset = match self.placement {
                    Placement::VerticalLeft => printer.size.x - 1,
                    _ => 0,
                };
                printer.print_vline((horizontal_offset, 0), printer.size.y, "│");
                // Spacing for padding & crop end
                let inner_printer = printer
                    // Alignment
                    .offset((
                        0,
                        self.align
                            .get_offset(self.bar_size.y + self.children.len() + 1, printer.size.y),
                    ));
                for (idx, child) in self.children.iter().enumerate() {
                    // There is no chainable api...
                    let mut rel_sizes = self.sizes.clone();
                    rel_sizes.truncate(idx);
                    let mut print = inner_printer
                        .offset(
                            rel_sizes
                                .iter()
                                .fold(Vec2::new(0, 0), |acc, x| acc.stack_vertical(x))
                                .keep_y(),
                        )
                        // Spacing for first character
                        .offset((0, idx))
                        .cropped({
                            if idx == 0 || idx == self.children.len() - 1 {
                                self.sizes[idx].stack_vertical(&Vec2::new(1, 2))
                            } else {
                                self.sizes[idx].stack_vertical(&Vec2::new(1, 1))
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
                                printer.print_hline((0, 0), printer.size.x, "━");
                            } else {
                                printer.print_hline((0, 0), printer.size.x, "─");
                            }
                        } else if child.active {
                            printer.print_hline((0, 0), printer.size.x, "━");
                            printer.print((horizontal_offset, 0), "┷")
                        } else {
                            printer.print_hline((0, 0), printer.size.x, "─");
                            printer.print((horizontal_offset, 0), "┴");
                        }
                        printer.with_effect(Effect::Bold, |printer| {
                            child.draw(&printer.offset((0, 1)))
                        });
                        if idx == self.children.len() - 1 {
                            let (delim, connector) = if child.active {
                                ("━", "┯")
                            } else {
                                ("─", "┬")
                            };
                            printer.print_hline((0, printer.size.y - 1), printer.size.x, delim);
                            printer.print(
                                self.sizes[idx].keep_y() + Vec2::new(horizontal_offset, 1),
                                connector,
                            );
                        }
                    });
                }
            }
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
        let placement = self.placement;
        let total_size = self
            .children
            .iter_mut()
            .fold(Vec2::zero(), |mut acc, child| {
                let size = child.required_size(cst);
                match placement {
                    Placement::HorizontalBottom | Placement::HorizontalTop => {
                        acc = acc.stack_horizontal(&size);
                    }
                    Placement::VerticalLeft | Placement::VerticalRight => {
                        acc = acc.stack_vertical(&size);
                    }
                }
                child.pos = acc;
                sizes.push(size);
                acc
            });
        // Total size of bar
        self.bar_size = total_size;
        // Return max width and maximum height of child
        match self.placement {
            Placement::VerticalRight | Placement::VerticalLeft => Vec2::new(
                // Maximum width
                self.sizes.iter().fold(0, |mut val, x| {
                    if val < x.x {
                        val = x.x;
                    }
                    val
                }),
                cst.y,
            ),
            Placement::HorizontalTop | Placement::HorizontalBottom => {
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
        }
    }

    fn on_event(&mut self, evt: Event) -> EventResult {
        if let Event::Mouse {
            offset,
            position,
            event,
        } = evt
        {
            for (idx, child) in self.children.iter().peekable().enumerate() {
                if position.checked_sub(offset).is_some()
                    && (match self.placement {
                        Placement::HorizontalBottom | Placement::HorizontalTop => {
                            child.pos
                                + Vec2::new(idx + 1, 0)
                                + Vec2::new(
                                    self.align.get_offset(
                                        // Length of buttons and delimiting characters
                                        self.bar_size.x + self.children.len() + 1,
                                        self.last_rendered_size.x,
                                    ),
                                    0,
                                )
                        }
                        Placement::VerticalLeft | Placement::VerticalRight => {
                            child.pos
                                + Vec2::new(0, idx + 1)
                                + Vec2::new(
                                    0,
                                    self.align.get_offset(
                                        // Length of buttons and delimiting characters
                                        self.bar_size.y + self.children.len() + 1,
                                        self.last_rendered_size.y,
                                    ),
                                )
                        }
                    })
                    .fits(position - offset)
                {
                    if let MouseEvent::Release(MouseButton::Left) = event {
                        self.invalidated = true;
                        self.idx = Some(idx);
                        return self.children[idx].on_event(Event::Key(Key::Enter));
                    }
                }
            }
        }

        if let Some(focus) = self.idx {
            let pos = self.children[focus].pos;

            if let EventResult::Consumed(any) = self.children[focus].on_event(evt.relativized(pos))
            {
                self.invalidated = true;
                return EventResult::Consumed(any);
            }
        }

        match evt {
            Event::Key(Key::Left)
                if self.placement == Placement::HorizontalBottom
                    || self.placement == Placement::HorizontalTop =>
            {
                self.decrement_idx()
            }
            Event::Key(Key::Up)
                if self.placement == Placement::VerticalLeft
                    || self.placement == Placement::VerticalRight =>
            {
                self.decrement_idx()
            }
            Event::Key(Key::Right)
                if self.placement == Placement::HorizontalBottom
                    || self.placement == Placement::HorizontalTop =>
            {
                self.increment_idx()
            }
            Event::Key(Key::Down)
                if self.placement == Placement::VerticalLeft
                    || self.placement == Placement::VerticalRight =>
            {
                self.increment_idx()
            }
            _ => EventResult::Ignored,
        }
    }
}
