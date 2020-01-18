use crossbeam::{unbounded, Receiver, Sender};
use cursive::direction::{Absolute, Direction};
use cursive::event::{AnyCb, Event, EventResult, Key};
use cursive::view::{Selector, View};
use cursive::{Printer, Vec2};
use log::debug;
use num::clamp;
use std::fmt::Display;
use std::hash::Hash;

use crate::Bar;
use crate::TabBar;
use crate::TabView;

#[derive(Clone, Copy, Debug)]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    VerticalLeft,
    VerticalRight,
    HorizontalTop,
    HorizontalBottom,
}

impl Align {
    pub fn get_offset(&self, content: usize, container: usize) -> usize {
        if container < content {
            0
        } else {
            match *self {
                Align::Start => 0,
                Align::Center => (container - content) / 2,
                Align::End => container - content,
            }
        }
    }
}

/// The `TabPanel` is an ease of use wrapper around a `TabView` and its `TabBar`.
/// Additionally the TabBar in the Panel can be horizontally aligned, by default it is set to be left aligned.
///
/// # Example
/// ```
/// use cursive_tabs::{Align, TabPanel};
/// use cursive::views::TextView;
///
/// let mut tabs = TabPanel::new()
///       .with_tab("First", TextView::new("First"))
///       .with_tab("Second", TextView::new("Second"))
///       .with_bar_alignment(Align::Center);
/// ```
///
/// A TabView is also usable separately, so if you prefer the tabs without the TabBar and Panel around have a look at `TabView`.
pub struct TabPanel<K: Hash + Eq + Display + Copy + 'static> {
    order: Vec<K>,
    bar: TabBar<K>,
    bar_size: Vec2,
    tab_size: Vec2,
    tx: Sender<K>,
    tabs: TabView<K>,
    active_rx: Receiver<K>,
    bar_focused: bool,
    bar_align: Align,
    bar_placement: Placement,
}

impl<K: Hash + Eq + Copy + Display + 'static> TabPanel<K> {
    /// Returns a new instance of a TabPanel.
    /// Alignment is set by default to left, to change this use `set_bar_alignment` to change to any other `HAlign` provided by `cursive`.
    pub fn new() -> Self {
        let mut tabs = TabView::new();
        let (tx, rx) = unbounded();
        let (active_tx, active_rx) = unbounded();
        tabs.set_bar_rx(rx);
        tabs.set_active_key_tx(active_tx);
        Self {
            order: Vec::new(),
            bar: TabBar::new(active_rx.clone()),
            bar_size: Vec2::new(1, 1),
            tab_size: Vec2::new(1, 1),
            tabs,
            tx,
            active_rx,
            bar_focused: true,
            bar_align: Align::Start,
            bar_placement: Placement::HorizontalTop,
        }
    }

    /// Returns the current active tab of the `TabView`.
    /// Note: Calls `active_tab` on the enclosed `TabView`.
    pub fn active_tab(&self) -> Option<K> {
        self.tabs.active_tab()
    }

    /// Non-consuming variant to set the active tab in the `TabView`.
    /// Note: Calls `set_active_tab` on the enclosed `TabView`.
    pub fn set_active_tab(&mut self, id: K) -> Result<(), ()> {
        self.tabs.set_active_tab(id)
    }

    /// Consuming & Chainable variant to set the active tab in the `TabView`.
    ///  Note: Calls `set_active_tab` on the enclosed `TabView`.
    ///
    /// Be careful! Failure in this case means the panel get dropped this has to with some trait restrictions in cursive, at the moment just avoid using the chainable variant if you are unsure that the operation will succeed.
    pub fn with_active_tab(mut self, id: K) -> Result<Self, ()> {
        // TODO: Return Self in error case, this is borked as of now
        self.tabs.set_active_tab(id)?;
        Ok(self)
    }

    /// Non-consuming variant to add new tabs to the `TabView`.
    /// Note: Calls `add_tab` on the enclosed `TabView`.
    pub fn add_tab<T: View>(&mut self, id: K, view: T) {
        self.tabs.add_tab(id, view);
    }

    /// Consuming & Chainable variant to add a new tab.
    /// Note: Calls `add_tab` on the enclosed `TabView`.
    pub fn with_tab<T: View>(mut self, id: K, view: T) -> Self {
        self.tabs.add_tab(id, view);

        self
    }

    /// Swaps the given tab keys.
    /// If at least one of them cannot be found then no operation is performed
    pub fn swap_tabs(&mut self, fst: K, snd: K) {
        self.tabs.swap_tabs(fst, snd);
    }

    /// Non-consuming variant to add new tabs to the `TabView` at a certain position.
    /// It is fail-safe, if the postion is greater than the amount of tabs, it is appended to the end.
    /// Note: Calls `add_tab_at` on the enclosed `TabView`.
    pub fn add_tab_at<T: View>(&mut self, id: K, view: T, pos: usize) {
        self.tabs.add_tab_at(id, view, pos);
    }

    /// Consuming & Chainable variant to add a new tab at a certain position.
    /// It is fail-safe, if the postion is greater than the amount of tabs, it is appended to the end.
    /// Note: Calls `add_tab_at` on the enclosed `TabView`.
    pub fn with_tab_at<T: View>(mut self, id: K, view: T, pos: usize) -> Self {
        self.tabs.add_tab_at(id, view, pos);

        self
    }

    /// Remove a tab of the enclosed `TabView`.
    pub fn remove_tab(&mut self, id: K) -> Result<(), ()> {
        self.tabs.remove_tab(id)
    }

    /// Proceeds to the next view in order of addition.
    pub fn next(&mut self) {
        self.tabs.next()
    }

    /// Go back to the previous view in order of addition.
    pub fn prev(&mut self) {
        self.tabs.prev()
    }

    /// Consumable & Chainable variant to set the bar alignment.
    pub fn with_bar_alignment(mut self, align: Align) -> Self {
        self.set_bar_alignment(align);

        self
    }

    /// Non-consuming variant to set the bar alignment.
    pub fn set_bar_alignment(&mut self, align: Align) {
        self.bar_align = align;
    }

    pub fn with_bar_placement(mut self, placement: Placement) -> Self {
        self.set_bar_placement(placement);
        self
    }

    pub fn set_bar_placement(&mut self, placement: Placement) {
        self.bar_placement = placement;
    }

    /// Returns the current order of tabs as an Vector with the keys of the views.
    pub fn tab_order(&self) -> Vec<K> {
        self.tabs.tab_order()
    }

    // Print lines corresponding to the current placement
    fn draw_outer_panel(&self, printer: &Printer) {
        match self.bar_placement {
            Placement::HorizontalTop => {
                // Side bars
                printer.print_vline((0, 0), printer.size.y, "│");
                printer.print_vline((printer.size.x - 1, 0), printer.size.y, "│");
                // Bottom line
                printer.print_hline((0, printer.size.y - 1), printer.size.x, "─");

                printer.print((0, self.bar_size.y - 1), "┌");
                printer.print((printer.size.x - 1, self.bar_size.y - 1), "┐");
                printer.print((0, printer.size.y - 1), "└");
                printer.print((printer.size.x - 1, printer.size.y - 1), "┘");
            }
            Placement::HorizontalBottom => {
                // Side bars
                printer.print_vline((0, 0), printer.size.y, "│");
                printer.print_vline((printer.size.x - 1, 0), printer.size.y, "│");
                // Top line
                let lowest = clamp(printer.size.y - self.bar_size.y, 0, printer.size.y - 1);
                printer.print_hline((0, 0), printer.size.x, "─");
                printer.print((0, 0), "┌");
                printer.print((printer.size.x - 1, 0), "┐");
                printer.print((0, lowest), "└");
                printer.print((printer.size.x - 1, lowest), "┘");
            }
            Placement::VerticalLeft => {
                // Side bar
                printer.print_vline((printer.size.x - 1, 0), printer.size.y, "│");
                // Top lines
                printer.print_hline((self.bar_size.x - 1, 0), printer.size.x, "─");
                printer.print_hline(
                    (self.bar_size.x - 1, printer.size.y - 1),
                    printer.size.x,
                    "─",
                );
                printer.print((self.bar_size.x - 1, 0), "┌");
                printer.print((printer.size.x - 1, 0), "┐");
                printer.print((self.bar_size.x - 1, printer.size.y - 1), "└");
                printer.print((printer.size.x - 1, printer.size.y - 1), "┘");
            }
            Placement::VerticalRight => {
                // Side bar
                printer.print_vline((0, 0), printer.size.y, "│");
                // Top lines
                printer.print_hline((0, 0), printer.size.x, "─");
                // Line draws too far here, needs to be overwritten with blanks
                printer.print_hline((0, printer.size.y - 1), printer.size.x, "─");

                let right = clamp(printer.size.x - self.bar_size.x, 0, printer.size.x - 1);
                printer.print((0, 0), "┌");
                printer.print((right, 0), "┐");
                printer.print_hline((right + 1, 0), printer.size.x, " ");
                printer.print((0, printer.size.y - 1), "└");
                printer.print((right, printer.size.y - 1), "┘");
                printer.print_hline((right + 1, printer.size.y - 1), printer.size.x, " ");
            }
        }
    }
}

impl<K: Hash + Eq + Copy + std::fmt::Display + 'static> View for TabPanel<K> {
    fn draw(&self, printer: &Printer) {
        self.draw_outer_panel(printer);
        let printer_bar = printer
            .offset(match self.bar_placement {
                Placement::HorizontalTop => (1, 0),
                Placement::HorizontalBottom => (
                    1,
                    clamp(printer.size.y - self.bar_size.y, 0, printer.size.y - 1),
                ),
                Placement::VerticalLeft => (0, 1),
                Placement::VerticalRight => (
                    clamp(printer.size.x - self.bar_size.x, 0, printer.size.x - 1),
                    1,
                ),
            })
            .cropped(match self.bar_placement {
                Placement::HorizontalTop | Placement::HorizontalBottom => {
                    (printer.size.x - 2, self.bar_size.y)
                }
                Placement::VerticalRight | Placement::VerticalLeft => {
                    (self.bar_size.x, printer.size.y - 2)
                }
            })
            .focused(self.bar_focused);
        let printer_tab = printer
            .offset(match self.bar_placement {
                Placement::VerticalLeft => (self.bar_size.x, 1),
                Placement::VerticalRight => (1, 1),
                Placement::HorizontalBottom => (1, 1),
                Placement::HorizontalTop => (1, self.bar_size.y),
            })
            // Inner area
            .cropped(match self.bar_placement {
                Placement::VerticalLeft | Placement::VerticalRight => {
                    (printer.size.x - self.bar_size.x - 1, printer.size.y - 2)
                }
                Placement::HorizontalBottom | Placement::HorizontalTop => {
                    (printer.size.x - 2, printer.size.y - self.bar_size.y - 1)
                }
            })
            .focused(!self.bar_focused);
        self.bar.draw(&printer_bar);
        self.tabs.draw(&printer_tab);
    }

    fn layout(&mut self, vec: Vec2) {
        self.bar.layout(match self.bar_placement {
            Placement::VerticalRight | Placement::VerticalLeft => {
                Vec2::new(self.bar_size.x, vec.y - 2)
            }
            Placement::HorizontalBottom | Placement::HorizontalTop => {
                Vec2::new(vec.x - 2, self.bar_size.y)
            }
        });
        self.tabs.layout(match self.bar_placement {
            Placement::VerticalRight | Placement::VerticalLeft => {
                self.tab_size = Vec2::new(vec.x - self.bar_size.x - 1, vec.y - 2);
                self.tab_size
            }
            Placement::HorizontalBottom | Placement::HorizontalTop => {
                self.tab_size = Vec2::new(vec.x - 2, vec.y - self.bar_size.y - 1);
                self.tab_size
            }
        });
    }

    fn needs_relayout(&self) -> bool {
        self.bar.needs_relayout() || self.tabs.needs_relayout()
    }

    fn required_size(&mut self, cst: Vec2) -> Vec2 {
        if self.order != self.tab_order() {
            debug!("Rebuilding TabBar");
            self.bar = TabBar::new(self.active_rx.clone())
                .with_alignment(self.bar_align)
                .with_placement(self.bar_placement);
            for key in self.tab_order() {
                self.bar.add_button(self.tx.clone(), key);
            }
            self.order = self.tab_order();
        }
        let tab_size = self.tabs.required_size(cst);
        self.bar_size = self.bar.required_size(cst);
        match self.bar_placement {
            Placement::HorizontalTop | Placement::HorizontalBottom => self
                .bar_size
                .stack_vertical(&tab_size)
                .stack_vertical(&Vec2::new(tab_size.x + 2, 1)),
            Placement::VerticalLeft | Placement::VerticalRight => self
                .bar_size
                .stack_horizontal(&tab_size)
                .stack_vertical(&Vec2::new(1, tab_size.y + 2)),
        }
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
                match self.bar_placement {
                    Placement::VerticalRight | Placement::HorizontalBottom => {
                        if position > offset && self.tab_size.fits(position - offset) {
                            if self.tabs.take_focus(Direction::front()) {
                                self.bar_focused = false;
                            }
                        } else {
                            self.bar_focused = true;
                        }
                    }
                    Placement::HorizontalTop | Placement::VerticalLeft => {
                        // We need to compare each individual value since the comparions with `>=` on the XY struct dPartialOrdoes not work as expected. When one value is the same but the other not it does not match although it should.
                        if position.x >= offset.x
                            && position.y >= offset.y
                            && (self.bar_size - Vec2::new(1, 1)).fits(position - offset)
                        {
                            self.bar_focused = true;
                        } else {
                            if self.tabs.take_focus(Direction::front()) {
                                self.bar_focused = false;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        if self.bar_focused {
            match self
                .bar
                .on_event(evt.clone().relativized(match self.bar_placement {
                    Placement::HorizontalTop | Placement::VerticalLeft => Vec2::new(0, 0),
                    Placement::HorizontalBottom => self.tab_size.keep_y() + Vec2::new(0, 1),
                    Placement::VerticalRight => self.tab_size.keep_x() + Vec2::new(1, 0),
                })) {
                EventResult::Consumed(cb) => EventResult::Consumed(cb),
                EventResult::Ignored => match evt {
                    Event::Key(Key::Down) if self.bar_placement == Placement::HorizontalTop => {
                        match self.tabs.take_focus(Direction::up()) {
                            true => {
                                self.bar_focused = false;
                                EventResult::Consumed(None)
                            }
                            false => EventResult::Ignored,
                        }
                    }
                    Event::Key(Key::Up) if self.bar_placement == Placement::HorizontalBottom => {
                        match self.tabs.take_focus(Direction::down()) {
                            true => {
                                self.bar_focused = false;
                                EventResult::Consumed(None)
                            }
                            false => EventResult::Ignored,
                        }
                    }
                    Event::Key(Key::Left) if self.bar_placement == Placement::VerticalRight => {
                        match self.tabs.take_focus(Direction::right()) {
                            true => {
                                self.bar_focused = false;
                                EventResult::Consumed(None)
                            }
                            false => EventResult::Ignored,
                        }
                    }
                    Event::Key(Key::Right) if self.bar_placement == Placement::VerticalLeft => {
                        match self.tabs.take_focus(Direction::left()) {
                            true => {
                                self.bar_focused = false;
                                EventResult::Consumed(None)
                            }
                            false => EventResult::Ignored,
                        }
                    }
                    _ => EventResult::Ignored,
                },
            }
        } else {
            match self
                .tabs
                .on_event(evt.relativized(match self.bar_placement {
                    Placement::HorizontalTop => Vec2::new(1, self.bar_size.y),
                    Placement::VerticalLeft => Vec2::new(self.bar_size.x, 1),
                    Placement::HorizontalBottom | Placement::VerticalRight => Vec2::new(1, 1),
                })) {
                EventResult::Consumed(cb) => EventResult::Consumed(cb),
                EventResult::Ignored => match evt {
                    Event::Key(Key::Up) if self.bar_placement == Placement::HorizontalTop => {
                        self.bar_focused = true;
                        EventResult::Consumed(None)
                    }
                    Event::Key(Key::Down) if self.bar_placement == Placement::HorizontalBottom => {
                        self.bar_focused = true;
                        EventResult::Consumed(None)
                    }
                    Event::Key(Key::Left) if self.bar_placement == Placement::VerticalLeft => {
                        self.bar_focused = true;
                        EventResult::Consumed(None)
                    }
                    Event::Key(Key::Right) if self.bar_placement == Placement::VerticalRight => {
                        self.bar_focused = true;
                        EventResult::Consumed(None)
                    }
                    _ => EventResult::Ignored,
                },
            }
        }
    }

    fn take_focus(&mut self, d: Direction) -> bool {
        let tabs_take_focus = |panel: &mut TabPanel<K>, d: Direction| {
            if panel.tabs.take_focus(d) {
                panel.bar_focused = false;
            } else {
                panel.bar_focused = true;
            }
        };

        match self.bar_placement {
            Placement::HorizontalBottom => match d {
                Direction::Abs(Absolute::Up) => tabs_take_focus(self, d),
                Direction::Abs(Absolute::Left) | Direction::Abs(Absolute::Right) => {
                    if !self.bar_focused {
                        tabs_take_focus(self, d)
                    }
                }
                Direction::Abs(Absolute::Down) => self.bar_focused = true,
                _ => {}
            },
            Placement::HorizontalTop => match d {
                Direction::Abs(Absolute::Down) => tabs_take_focus(self, d),
                Direction::Abs(Absolute::Left) | Direction::Abs(Absolute::Right) => {
                    if !self.bar_focused {
                        tabs_take_focus(self, d)
                    }
                }
                Direction::Abs(Absolute::Up) => self.bar_focused = true,
                _ => {}
            },
            Placement::VerticalLeft => match d {
                Direction::Abs(Absolute::Right) => tabs_take_focus(self, d),
                Direction::Abs(Absolute::Up) | Direction::Abs(Absolute::Down) => {
                    if !self.bar_focused {
                        tabs_take_focus(self, d)
                    }
                }
                Direction::Abs(Absolute::Left) => self.bar_focused = true,
                _ => {}
            },
            Placement::VerticalRight => match d {
                Direction::Abs(Absolute::Left) => tabs_take_focus(self, d),
                Direction::Abs(Absolute::Up) | Direction::Abs(Absolute::Down) => {
                    if !self.bar_focused {
                        tabs_take_focus(self, d)
                    }
                }
                Direction::Abs(Absolute::Right) => self.bar_focused = true,
                _ => {}
            },
        }
        true
    }

    fn focus_view(&mut self, slt: &Selector) -> Result<(), ()> {
        self.tabs.focus_view(slt)
    }

    fn call_on_any<'a>(&mut self, slt: &Selector, cb: AnyCb<'a>) {
        self.bar.call_on_any(slt, cb);
        self.tabs.call_on_any(slt, cb);
    }
}
