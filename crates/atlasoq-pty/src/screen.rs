//! Extração de texto da tela e do scrollback a partir do parser headless.

/// Últimas `lines` linhas (scrollback + tela), sem formatação e sem linhas vazias no fim.
pub(crate) fn last_lines(parser: &mut vt100::Parser, lines: usize) -> String {
    let (rows, cols) = parser.screen().size();
    let rows = rows as usize;
    let screen = parser.screen_mut();
    screen.set_scrollback(usize::MAX);
    let total_scrollback = screen.scrollback();

    let mut all: Vec<String> = Vec::with_capacity(total_scrollback + rows);
    let mut start = 0;
    while start < total_scrollback {
        let offset = total_scrollback - start;
        screen.set_scrollback(offset);
        let take = rows.min(offset);
        all.extend(screen.rows(0, cols).take(take));
        start += take;
    }
    screen.set_scrollback(0);
    all.extend(screen.rows(0, cols));

    for line in &mut all {
        let trimmed = line.trim_end().len();
        line.truncate(trimmed);
    }
    while all.last().is_some_and(|l| l.is_empty()) {
        all.pop();
    }
    let skip = all.len().saturating_sub(lines);
    all[skip..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inclui_scrollback_e_tela() {
        let mut p = vt100::Parser::new(3, 20, 100);
        for i in 0..10 {
            p.process(format!("linha {i}\r\n").as_bytes());
        }
        let text = last_lines(&mut p, 100);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.first(), Some(&"linha 0"));
        assert_eq!(lines.last(), Some(&"linha 9"));
        assert_eq!(lines.len(), 10);
        assert_eq!(last_lines(&mut p, 2), "linha 8\nlinha 9");
        assert_eq!(p.screen().scrollback(), 0);
    }

    #[test]
    fn remove_ansi() {
        let mut p = vt100::Parser::new(5, 40, 10);
        p.process(b"\x1b[31mvermelho\x1b[0m normal");
        assert_eq!(last_lines(&mut p, 10), "vermelho normal");
    }
}
