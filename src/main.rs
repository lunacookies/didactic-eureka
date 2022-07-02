use std::io::{self, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;

        stdin.read_line(&mut input)?;

        if input.is_empty() {
            break;
        }

        let tokens = dbg!(didactic_eureka::lexer::lex(&input));
        let items = dbg!(didactic_eureka::parser::parse(&tokens));
        let _ = items;

        input.clear();
    }

    Ok(())
}
