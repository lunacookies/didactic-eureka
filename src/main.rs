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

		let tokens = didactic_eureka::lexer::lex(&input);
		dbg!(&tokens);

		let source_file = didactic_eureka::parser::parse(&tokens);
		dbg!(&source_file);
		let block = didactic_eureka::tac::lower(&source_file);
		dbg!(&block);

		input.clear();
	}

	Ok(())
}
