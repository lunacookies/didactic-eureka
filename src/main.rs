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

        match analyze(&input) {
            Ok(()) => {}
            Err(e) => didactic_eureka::errors::print_error(&e, &input),
        }

        input.clear();
    }

    Ok(())
}

fn analyze(input: &str) -> Result<(), didactic_eureka::errors::Error> {
    let tokens = dbg!(didactic_eureka::lexer::lex(input));
    let items = dbg!(didactic_eureka::parser::parse(&tokens)?);

    let index = dbg!(didactic_eureka::index::index(&items)?);

    for item in &items {
        if let didactic_eureka::ast::ItemKind::Function(f) = &item.kind {
            let (block, db) = didactic_eureka::body::lower(f, &index)?;
            dbg!(&block, &db);
        }
    }

    Ok(())
}
