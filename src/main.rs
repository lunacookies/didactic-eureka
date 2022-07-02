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

        dbg!(didactic_eureka::lex(&input));

        input.clear();
    }

    Ok(())
}
