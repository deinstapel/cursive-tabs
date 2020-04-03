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

/// Trait which defines which basic action a tab bar should be able to handle
pub trait Bar<K: Hash + Eq + Copy + Display + 'static> {
    fn add_button(&mut self, tx: Sender<K>, key: K);
    fn remove_button(&mut self, key: K);
    fn swap_button(&mut self, left: K, right: K);
    fn add_button_at(&mut self, tx: Sender<K>, key: K, pos: usize);
}

// Quick Wrapper around Views to be able to set their positon
struct PositionWrap<T: View, K> {
    view: T,
    pub pos: Vec2,
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
    cursor: Option<usize>,
    active: Option<usize>,
    rx: Receiver<K>,
    invalidated: bool,
}

impl<K: Hash + Eq + Copy + Display + 'static> TabBar<K> {
    pub fn new(rx: Receiver<K>) -> Self {
        Self {
            children: Vec::new(),
            sizes: Vec::new(),
            cursor: None,
            active: None,
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

    pub fn set_alignment(&mut self, align: Align) {
        self.align = align;
        self.invalidated = true;
    }

    pub fn with_placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self.invalidated = true;
        self
    }

    pub fn set_placement(&mut self, placement: Placement) {
        self.placement = placement;
        self.invalidated = true;
    }

    fn decrement_idx(&mut self) -> EventResult {
        if let Some(index) = self.cursor {
            if index > 0 {
                self.cursor = Some(index - 1);
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
        if let Some(index) = self.cursor {
            if (index + 1) < self.children.len() {
                self.cursor = Some(index + 1);
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
        self.cursor = Some(self.children.len() - 1);
        self.active = Some(self.children.len() - 1);
        self.invalidated = true;
    }

    fn remove_button(&mut self, key: K) {
        if let Some(pos) = self
            .children
            .iter()
            .enumerate()
            .filter_map(
                |(pos, button)| {
                    if button.key == key {
                        Some(pos)
                    } else {
                        None
                    }
                },
            )
            .next()
        {
            if let Some(idx) = self.cursor {
                if idx == pos {
                    self.cursor = None;
                    self.active = None;
                }
            }
            self.children.remove(pos);
        }
        self.invalidated = true;
    }

    fn swap_button(&mut self, first: K, second: K) {
        let pos: Vec<usize> = self
            .children
            .iter()
            .enumerate()
            .filter_map(|(pos, button)| {
                if button.key == first || button.key == second {
                    Some(pos)
                } else {
                    None
                }
            })
            .collect();
        if let [pos1, pos2] = pos[..] {
            let child2 = self.children.remove(pos2);
            let child1 = self.children.remove(pos1);
            self.children.insert(pos1, child2);
            self.children.insert(pos2, child1);
        }
        self.invalidated = true;
    }

    fn add_button_at(&mut self, tx: Sender<K>, key: K, pos: usize) {
        let button = Button::new_raw(format!(" {} ", key), move |_| {
            debug!("send {}", key);
            match tx.send(key) {
                Ok(_) => {}
                Err(err) => {
                    debug!("button could not send key: {:?}", err);
                }
            }
        });
        self.cursor = Some(pos);
        self.active = Some(pos);
        self.children.insert(pos, PositionWrap::new(button, key));
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

                    if !self.active.map_or(false, |active| idx == active) {
                        let color = theme.palette[PaletteColor::TitleSecondary];
                        theme.palette[PaletteColor::Primary] = color;
                    } else {
                        let color = theme.palette[PaletteColor::TitlePrimary];
                        theme.palette[PaletteColor::Primary] = color;
                    }

                    if let Some(focus) = self.cursor {
                        print = print.focused(focus == idx);
                    }

                    print.with_theme(&theme, |printer| {
                        if idx > 0 {
                            if self.active.map_or(false, |active| idx == active)
                                || self.active.map_or(false, |active| active == (idx - 1))
                            {
                                printer.print((0, 0), "┃")
                            } else {
                                printer.print((0, 0), "│");
                            }
                        } else if self.active.map_or(false, |active| idx == active) {
                            printer.print((0, 0), "┨")
                        } else {
                            printer.print((0, 0), "┤");
                        }
                        printer.with_effect(Effect::Bold, |printer| {
                            child.draw(&printer.offset((1, 0)))
                        });
                        if idx == self.children.len() - 1 {
                            if self.active.map_or(false, |active| idx == active) {
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

                    if !self.active.map_or(false, |active| idx == active) {
                        let color = theme.palette[PaletteColor::TitleSecondary];
                        theme.palette[PaletteColor::Primary] = color;
                    } else {
                        let color = theme.palette[PaletteColor::TitlePrimary];
                        theme.palette[PaletteColor::Primary] = color;
                    }

                    if let Some(focus) = self.cursor {
                        print = print.focused(focus == idx);
                    }
                    print.with_theme(&theme, |printer| {
                        if idx > 0 {
                            if self.active.map_or(false, |active| idx == active)
                                || self.active.map_or(false, |active| active == (idx - 1))
                            {
                                printer.print_hline((0, 0), printer.size.x, "━");
                            } else {
                                printer.print_hline((0, 0), printer.size.x, "─");
                            }
                        } else if self.active.map_or(false, |active| idx == active) {
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
                            let (delim, connector) =
                                if self.active.map_or(false, |active| idx == active) {
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
        while (self.rx.len() > 1) {
            // Discard old messages
            // This may happen if more than one view gets added to before the event loop of cursive gets started, resulting
            // in an incorrect start state
            self.rx.try_recv();
        }
        if let Ok(new_active) = self.rx.try_recv() {
            self.invalidated = true;
            for (idx, child) in self.children.iter().enumerate() {
                if new_active == child.key {
                    self.active = Some(idx);
                }
            }
        }
        self.sizes.clear();
        let sizes = &mut self.sizes;
        let placement = self.placement;
        if self.children.is_empty() {
            return Vec2::new(1, 1);
        }
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
                        self.cursor = Some(idx);
                        return self.children[idx].on_event(Event::Key(Key::Enter));
                    }
                }
            }
        }

        if let Some(focus) = self.cursor {
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
