use crossbeam::{unbounded, Receiver, Sender};
use cursive::align::HAlign;
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

/// The `TabPanel` is an ease of use wrapper around a `TabView` and its `TabBar`.
/// Additionally the TabBar in the Panel can be horizontally aligned, by default it is set to be left aligned.
///
/// # Example
/// ```
/// use cursive_tabs::TabPanel;
/// use cursive::views::TextView;
/// use cursive::align::HAlign;
///
/// let mut tabs = TabPanel::new()
///       .with_tab("First", TextView::new("First"))
///       .with_tab("Second", TextView::new("Second"))
///       .with_bar_alignment(HAlign::Center);
/// ```
///
/// A TabView is also usable separately, so if you prefer the tabs without the TabBar and Panel around have a look at `TabView`.
pub struct TabPanel<K: Hash + Eq + Display + Copy + 'static> {
    order: Vec<K>,
    bar: TabBar<K>,
    bar_size: Vec2,
    tx: Sender<K>,
    tabs: TabView<K>,
    active_rx: Receiver<K>,
    bar_focused: bool,
    bar_h_align: HAlign,
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
            tabs,
            tx,
            active_rx,
            bar_focused: false,
            bar_h_align: HAlign::Left,
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
    pub fn with_bar_alignment(mut self, align: HAlign) -> Self {
        self.set_bar_alignment(align);

        self
    }

    /// Non-consuming variant to set the bar alignment.
    pub fn set_bar_alignment(&mut self, align: HAlign) {
        self.bar_h_align = align;
    }

    /// Returns the current order of tabs as an Vector with the keys of the views.
    pub fn tab_order(&self) -> Vec<K> {
        self.tabs.tab_order()
    }

    // Workaround, neither clone or copy implemented for HAlign
    fn clone_align(align: &HAlign) -> HAlign {
        match align {
            HAlign::Center => HAlign::Center,
            HAlign::Left => HAlign::Left,
            HAlign::Right => HAlign::Right,
        }
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
            // Retract panel size
            .layout(Vec2::new(vec.x - 2, vec.y - self.bar_size.y - 1));
    }

    fn needs_relayout(&self) -> bool {
        self.bar.needs_relayout() || self.tabs.needs_relayout()
    }

    fn required_size(&mut self, cst: Vec2) -> Vec2 {
        if self.order != self.tab_order() {
            debug!("Rebuilding TabBar");
            self.bar =
                TabBar::new(self.active_rx.clone()).h_align(Self::clone_align(&self.bar_h_align));
            for key in self.tab_order() {
                self.bar.add_button(self.tx.clone(), key);
            }
            self.order = self.tab_order();
        }
        let tab_size = self.tabs.required_size(cst);
        self.bar_size = self.bar.required_size(cst);
        self.bar_size
            .stack_vertical(&tab_size)
            // Offset for box drawing characters
            .stack_vertical(&Vec2::new(tab_size.x + 2, 1))
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
            match self.bar.on_event(evt.clone()) {
                EventResult::Consumed(cb) => EventResult::Consumed(cb),
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
                EventResult::Consumed(cb) => EventResult::Consumed(cb),
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
        self.bar.call_on_any(slt, Box::new(|any| cb(any)));
        self.tabs.call_on_any(slt, Box::new(|any| cb(any)));
    }
}
