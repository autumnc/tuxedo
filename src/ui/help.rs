use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::theme::Theme;

type Section = (&'static str, &'static [(&'static str, &'static str)]);

const NAVIGATION: Section = (
    "NAVIGATION",
    &[
        ("j / ↓", "next task"),
        ("k / ↑", "previous task"),
        ("gg", "first task"),
        ("G", "last task"),
        ("Ctrl-d / Ctrl-u", "page down / up"),
    ],
);

const EDITING: Section = (
    "EDITING",
    &[
        ("n", "new task"),
        ("e", "edit line (normal)"),
        ("i", "edit line (insert)"),
        ("r", "reschedule task"),
        ("x", "toggle complete"),
        ("dd", "delete task"),
        ("p", "cycle priority A→B→C→·"),
        ("c", "add/remove context"),
        ("+", "add project"),
        ("yy", "copy line to clipboard"),
        ("yb", "copy body only"),
        ("N", "add/edit note"),
        ("u", "undo"),
    ],
);

const VIEW: Section = (
    "VIEW",
    &[
        ("/", "fuzzy search"),
        ("fp / fc", "filter project/context"),
        ("ff / fs", "saved filter pick/save"),
        ("S", "cycle sort"),
        ("v", "visual / multi-select"),
        ("l", "list view"),
        ("a", "archive view"),
        ("A", "archive completed"),
        ("H", "show done in list"),
        ("F", "show future in list"),
        ("[ / ]", "toggle filter / detail"),
        ("T", "cycle theme"),
        ("D", "cycle density"),
        ("L", "toggle line numbers"),
    ],
);

const SYSTEM: Section = (
    "SYSTEM",
    &[
        (": / Ctrl-P", "command palette"),
        ("s", "share capture QR"),
        ("? / ,", "help / settings"),
        ("q", "quit"),
    ],
);

const FORMAT: Section = (
    "FORMAT",
    &[
        ("(A)", "priority A-Z"),
        ("YYYY-MM-DD", "creation / done date"),
        ("+project", "project tag(s)"),
        ("@context", "context tag(s)"),
        ("due:YYYY-MM-DD", "due date"),
        ("rec:Nu", "recur (u in d/w/m/y/b)"),
        ("rec:+Nu", "strict: anchor on due:"),
        ("x DATE BODY", "completed task prefix"),
    ],
);

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border).bg(theme.panel))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "tuxedo",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" · help ".to_string(), Style::default().fg(theme.dim)),
        ]))
        .style(Style::default().bg(theme.panel));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let bg = Style::default().bg(theme.panel).fg(theme.fg);

    let mut content: Vec<Line> = Vec::new();
    let all: &[Section] = &[NAVIGATION, EDITING, VIEW, SYSTEM, FORMAT];
    content.extend(render_sections_trimmed(theme, all));

    // Scrolling.
    let content_h = inner.height as usize;
    let total = content.len().max(1);
    let max_scroll = total.saturating_sub(content_h);
    let scroll = (app.help_scroll.get() as usize).min(max_scroll);
    // Write the clamped scroll back so key handlers see a consistent value.
    app.help_scroll.set(scroll as u16);

    let visible: Vec<Line> = if total <= content_h {
        content
    } else {
        content[scroll..(scroll + content_h).min(total)].to_vec()
    };

    frame.render_widget(Paragraph::new(visible).style(bg), inner);
}

fn render_sections_trimmed<'a>(theme: &Theme, sections: &[Section]) -> Vec<Line<'a>> {
    let mut lines = render_sections(theme, sections);
    // Drop the trailing blank that `render_sections` appends after the last
    // section so columns end flush.
    if matches!(lines.last(), Some(line) if line_is_blank(line)) {
        lines.pop();
    }
    lines
}

fn line_is_blank(line: &Line) -> bool {
    line.spans
        .iter()
        .all(|s| s.content.chars().all(|c| c == ' '))
}

fn render_sections<'a>(theme: &Theme, sections: &[Section]) -> Vec<Line<'a>> {
    let mut lines: Vec<Line> = Vec::new();
    for (title, items) in sections {
        if title.is_empty() {
            lines.push(Line::raw(" "));
        } else {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    (*title).to_string(),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        for (k, d) in *items {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    pad_str(k, 18),
                    Style::default()
                        .fg(theme.context)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled((*d).to_string(), Style::default().fg(theme.fg)),
            ]));
        }
        lines.push(Line::raw(" "));
    }
    lines
}

fn pad_str(s: &str, w: usize) -> String {
    let len = s.chars().count();
    if len >= w {
        s.to_string()
    } else {
        let mut o = s.to_string();
        o.push_str(&" ".repeat(w - len));
        o
    }
}
