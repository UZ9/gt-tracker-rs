use std::io;

mod tui;

use gt_tracker_rs::course::Course;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    prelude::*,
    widgets::*,
};
use symbols::border;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default)]
pub struct App {
    state: TableState,
    courses: Vec<Course>,
    longest_item_lens: (u16, u16, u16, u16, u16),
    scroll_state: ScrollbarState,
    exit: bool,
}

const ITEM_HEIGHT: usize = 4;

impl App {
    fn new(courses: Vec<Course>) -> Self {
        Self {
            state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new((courses.len() - 1) * ITEM_HEIGHT),
            longest_item_lens: constraint_len_calculator(&courses),
            courses,
            exit: false,
        }
    }

    pub fn next(&mut self) {
        // Pagination handler
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.courses.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous(&mut self) {
        // Pagination handler
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    // Reached start, loop around
                    self.courses.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| ui(frame, self))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') => self.next(),
            KeyCode::Char('k') => self.previous(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

fn constraint_len_calculator(courses: &[Course]) -> (u16, u16, u16, u16, u16) {
    let name_len = courses
        .iter()
        .map(|course| course.name().as_str())
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let crn_length = 6;

    (name_len as u16, crn_length as u16, 3, 3, 3)
}

fn ui(frame: &mut Frame, app: &mut App) {
    let rects = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(frame.size());

    render_table(frame, app, rects[0]);

    render_scrollbar(frame, app, rects[0]);

    render_footer(frame, app, rects[1]);
}

fn render_footer(frame: &mut Frame, _: &mut App, area: Rect) {
    let info_footer = Paragraph::new(Line::from("(Esc) quit | (k) move up | (j) move down "))
        .style(Style::new().fg(style::Color::Yellow))
        .centered()
        .block(Block::bordered().border_set(border::THICK));

    frame.render_widget(info_footer, area);
}

fn render_scrollbar(frame: &mut Frame, app: &mut App, area: Rect) {
    frame.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );
}

fn determine_color(data: &Course) -> style::Color {
    return match data.class_enrollment().remaining() {
        0 => style::Color::Red,
        _ => style::Color::Green,
    };
}

fn render_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default();

    let header = ["Name", "CRN", "Capacity", "Actual", "Remaining"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    let rows = app.courses.iter().map(|data| {
        let item = data.ref_array();

        Row::new(item.iter().map(|content| {
            let color = determine_color(data);
            let text = Text::from(format!("\n{content}\n"));

            Cell::from(text).style(Style::new().fg(color))
        }))
        .height(4)
    });

    let bar = " ║ ";
    let t = Table::new(
        rows,
        [
            Constraint::Length(app.longest_item_lens.0 + 1),
            Constraint::Min(app.longest_item_lens.1 + 1),
            Constraint::Min(app.longest_item_lens.2),
            Constraint::Min(app.longest_item_lens.3),
            Constraint::Min(app.longest_item_lens.4),
        ],
    )
    .header(header)
    .highlight_symbol(Text::from(vec![
        "".into(),
        bar.into(),
        bar.into(),
        "".into(),
    ]))
    .block(Block::bordered().border_set(border::THICK))
    .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(t, area, &mut app.state);
}

fn main() -> io::Result<()> {
    let courses: Vec<Course> = gt_tracker_rs::get_input_courses();
    let mut terminal = tui::init()?;
    terminal.clear()?;
    let app_result = App::new(courses).run(&mut terminal);
    tui::restore()?;
    terminal.show_cursor()?;

    if let Err(err) = app_result {
        println!("{err:?}");
    }

    Ok(())
}
