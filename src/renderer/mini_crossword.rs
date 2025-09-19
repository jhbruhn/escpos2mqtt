use crate::mini_crossword;
use crate::printer::Command;
use escpos::utils::JustifyMode;
use unicode_width::UnicodeWidthStr;

fn format_list(s: &[String]) -> String {
    match s {
        [x] => x.clone(),
        [x, y] => [x, " and ", y].concat(),
        xs => {
            let mut out = String::new();
            for (i, x) in xs.iter().enumerate() {
                if i == xs.len() - 1 {
                    out.push_str(", and ");
                } else if i != 0 {
                    out.push_str(", ");
                }
                out.push_str(x);
            }
            out
        }
    }
}

pub fn make_mini_crossword() -> Vec<Command> {
    let cw = mini_crossword::get().expect("Could not get crossword");
    let puzzle = cw.puzzle;
    let wrap_opts = || textwrap::Options::new(42);

    let mut commands = vec![
        Command::ResetSize,
        Command::Write(cw.publication_date.strftime("%A, %B %-d, %Y").to_string() + "\n"),
        Command::Feed(1),
        Command::ResetSize,
        Command::Write(String::from("\n")),
        Command::Feed(1),
        Command::Justify(JustifyMode::CENTER),
        Command::BitImageFromBytes(cw.image),
        Command::Feed(2),
        Command::Justify(JustifyMode::LEFT),
    ];

    let write_wrapped = |text, opts: textwrap::Options<'_>| {
        let text = textwrap::wrap(text, opts);
        text.into_iter()
            .map(|line| Command::Write(format!("{}\n", line)))
            .collect::<Vec<Command>>()
    };

    for clues in &puzzle.clue_lists {
        commands.push(Command::Write(format!("{:?}:\n", clues.name)));
        for &clue_num in &clues.clues {
            let clue = &puzzle.clues[clue_num as usize];
            let label = format!("{}: ", clue.label);
            commands.extend(write_wrapped(
                &clue.text[0].plain,
                wrap_opts()
                    .initial_indent(&label)
                    .subsequent_indent(&" ".repeat(label.width())),
            ));
        }
        commands.push(Command::Feed(1));
    }

    commands.extend(write_wrapped(
        &format_list(&cw.constructors),
        wrap_opts().initial_indent("by ").subsequent_indent("   "),
    ));

    commands.push(Command::ResetSize);

    commands
}
