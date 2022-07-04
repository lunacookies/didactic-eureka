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

        match items {
            Ok(items) => {
                let index = dbg!(didactic_eureka::index::index(&items));

                for item in &items {
                    if let didactic_eureka::ast::Item::Function { body, .. } = item {
                        let mut ctx = didactic_eureka::body::LowerCtx::new(&index);
                        match dbg!(ctx.lower_block(body)) {
                            Ok(_) => {}
                            Err(e) => didactic_eureka::errors::print_error(&e, &input),
                        }
                    }
                }
            }
            Err(e) => didactic_eureka::errors::print_error(&e, &input),
        }

        input.clear();
    }

    Ok(())
}
