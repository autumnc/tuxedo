use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthChar;

use crate::app::App;
use crate::theme::Theme;
use crate::todo::Task;
use crate::ui::task_row::{due_label, due_token_style, is_url_token, url_token_style};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    super::fill_bg(frame, area, Style::default().bg(theme.panel));

    let task = app.cur_task();
    // Wrap to the actual pane width minus 1-char left padding and 1-char
    // safety margin on the right. Floor at 16 so a tiny pane still wraps.
    let wrap_w = (area.width as usize).saturating_sub(2).max(16);
    let lines = build_lines(theme, task, app.today(), wrap_w);
    let para = Paragraph::new(lines).style(Style::default().bg(theme.panel).fg(theme.fg));
    frame.render_widget(para, area);
}

fn build_lines<'a>(
    theme: &Theme,
    task: Option<&'a Task>,
    today: &'a str,
    wrap_w: usize,
) -> Vec<Line<'a>> {
    let mut rows: Vec<Line> = Vec::new();

    // ── DETAIL ──────────────────────────────────────────────────────────
    rows.push(section_header(theme, "DETAIL"));
    rows.push(line_panel(theme, vec![Span::raw(" ")]));
    let Some(t) = task else {
        rows.push(line_panel(
            theme,
            vec![Span::styled(" (no task)", Style::default().fg(theme.dim))],
        ));
        return rows;
    };

    let priority_value = if let Some(p) = t.priority {
        Span::styled(
            format!("({p})"),
            Style::default()
                .fg(theme.priority_color(p))
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
    };
    rows.push(line_panel(
        theme,
        vec![
            Span::styled(" priority  ", Style::default().fg(theme.dim)),
            priority_value,
        ],
    ));
    rows.push(line_panel(
        theme,
        vec![
            Span::styled(" created   ", Style::default().fg(theme.dim)),
            Span::styled(
                t.created_date.as_deref().unwrap_or("—"),
                Style::default().fg(theme.fg),
            ),
        ],
    ));
    if let Some(due) = &t.due {
        rows.push(line_panel(
            theme,
            vec![
                Span::styled(" due       ", Style::default().fg(theme.dim)),
                Span::styled(due.as_str(), Style::default().fg(theme.fg)),
                Span::raw("  "),
                Span::styled(due_label(due, today), Style::default().fg(theme.overdue)),
            ],
        ));
    }
    rows.push(line_panel(
        theme,
        vec![
            Span::styled(" projects  ", Style::default().fg(theme.dim)),
            Span::styled(
                t.projects
                    .iter()
                    .map(|p| format!("+{p}"))
                    .collect::<Vec<_>>()
                    .join(" "),
                Style::default().fg(theme.project),
            ),
        ],
    ));
    rows.push(line_panel(
        theme,
        vec![
            Span::styled(" contexts  ", Style::default().fg(theme.dim)),
            Span::styled(
                t.contexts
                    .iter()
                    .map(|c| format!("@{c}"))
                    .collect::<Vec<_>>()
                    .join(" "),
                Style::default().fg(theme.context),
            ),
        ],
    ));
    if t.done {
        rows.push(line_panel(
            theme,
            vec![
                Span::styled(" done      ", Style::default().fg(theme.dim)),
                Span::styled(
                    t.done_date.as_deref().unwrap_or(""),
                    Style::default().fg(theme.done),
                ),
            ],
        ));
    }

    // ── NOTES ───────────────────────────────────────────────────────────
    if !t.notes.is_empty() {
        rows.push(line_panel(theme, vec![Span::raw(" ")]));
        rows.push(section_header(theme, "NOTES"));
        rows.push(line_panel(theme, vec![Span::raw(" ")]));
        for note in &t.notes {
            let chunks = wrap_words(note, wrap_w.saturating_sub(2));
            for chunk in chunks {
                rows.push(line_panel(
                    theme,
                    vec![Span::styled(
                        format!(" {}", chunk.join(" ")),
                        Style::default().fg(theme.fg),
                    )],
                ))
            }
        }
    }

    // ── RAW ─────────────────────────────────────────────────────────────
    rows.push(line_panel(theme, vec![Span::raw(" ")]));
    rows.push(section_header(theme, "RAW"));
    rows.push(line_panel(theme, vec![Span::raw(" ")]));
    let mut state = RawWalk::default();
    for chunk in wrap_words(&t.clean_raw, wrap_w) {
        let mut spans: Vec<Span> = vec![Span::raw(" ")];
        let mut words = chunk.into_iter();
        if let Some(first) = words.next() {
            spans.push(style_raw_token(first, t, today, theme, &mut state));
        }
        for w in words {
            spans.push(Span::raw(" "));
            spans.push(style_raw_token(w, t, today, theme, &mut state));
        }
        rows.push(line_panel(theme, spans));
    }
    rows
}

fn section_header<'a>(theme: &Theme, label: &'static str) -> Line<'a> {
    Line::from(vec![Span::styled(
        format!(" {label}"),
        Style::default().fg(theme.dim).add_modifier(Modifier::BOLD),
    )])
    .style(Style::default().bg(theme.panel))
}

#[derive(Default)]
struct RawWalk {
    done_marker_consumed: bool,
    priority_consumed: bool,
}

fn style_raw_token<'a>(
    token: &'a str,
    task: &Task,
    today: &str,
    theme: &Theme,
    state: &mut RawWalk,
) -> Span<'a> {
    if task.done && !state.done_marker_consumed {
        state.done_marker_consumed = true;
        if token == "x" {
            return Span::styled(token, Style::default().fg(theme.done));
        }
    }
    if !state.priority_consumed
        && let Some(p) = task.priority
        && token.len() == 3
        && token.as_bytes()[0] == b'('
        && token.as_bytes()[1] == p as u8
        && token.as_bytes()[2] == b')'
    {
        state.priority_consumed = true;
        return Span::styled(
            token,
            Style::default()
                .fg(theme.priority_color(p))
                .add_modifier(Modifier::BOLD),
        );
    }
    if let Some(rest) = token.strip_prefix("due:") {
        return Span::styled(token, due_token_style(task.done, rest, today, theme));
    }
    if is_url_token(token) {
        return Span::styled(token, url_token_style(task.done, theme));
    }
    if token.len() > 1 && token.starts_with('+') {
        return Span::styled(token, Style::default().fg(theme.project));
    }
    if token.len() > 1 && token.starts_with('@') {
        return Span::styled(token, Style::default().fg(theme.context));
    }
    Span::styled(token, Style::default().fg(theme.fg))
}

fn line_panel<'a>(theme: &Theme, spans: Vec<Span<'a>>) -> Line<'a> {
    Line::from(spans).style(Style::default().bg(theme.panel))
}

/// Wrap `s` to `width` display columns. Each output line is a vector of
/// borrowed word slices. Display width uses `unicode_width` so CJK characters
/// count as 2 columns. A token wider than `width` (common in CJK text without
/// whitespace) is split character by character so it never overflows.
fn wrap_words(s: &str, width: usize) -> Vec<Vec<&str>> {
    let mut out: Vec<Vec<&str>> = Vec::new();
    let mut line: Vec<&str> = Vec::new();
    let mut line_w: usize = 0;

    for token in s.split_whitespace() {
        let token_w: usize = token.chars().map(|c| c.width().unwrap_or(0)).sum();
        let gap = if line.is_empty() { 0 } else { 1 };

        if token_w <= width {
            // Fits in a single line.
            if line_w + gap + token_w > width && !line.is_empty() {
                out.push(std::mem::take(&mut line));
                line_w = 0;
            }
            if !line.is_empty() {
                line_w += 1;
            }
            line.push(token);
            line_w += token_w;
        } else {
            // Token wider than `width` — split character by character.
            if !line.is_empty() {
                out.push(std::mem::take(&mut line));
                line_w = 0;
            }
            let width = width.max(1);
            let mut byte_start: usize = 0;
            let mut col: usize = 0;
            for (byte, c) in token.char_indices() {
                let cw = c.width().unwrap_or(0);
                if col + cw > width && col > 0 {
                    line.push(&token[byte_start..byte]);
                    out.push(std::mem::take(&mut line));
                    byte_start = byte;
                    col = 0;
                }
                col += cw;
            }
            if byte_start < token.len() {
                line.push(&token[byte_start..]);
                line_w = col;
            }
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}
