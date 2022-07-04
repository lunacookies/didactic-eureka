use std::ops::Range;

#[derive(Debug)]
pub struct Error {
    pub(crate) message: String,
    pub(crate) range: Range<usize>,
}

pub fn print_error(e: &Error, input: &str) {
    let mut newlines: Vec<_> = vec![0];
    for (idx, _) in input.match_indices('\n') {
        newlines.push(idx);
    }
    let line_nr = newlines.partition_point(|&i| i < e.range.start);
    let line_start = newlines[line_nr - 1];
    let line_end = newlines[line_nr];

    println!("\x1B[1;91merror\x1B[0m({line_nr}): \x1B[1m{}\x1B[0m", e.message);
    print!("    ");
    for (idx, c) in input[line_start..line_end].char_indices() {
        if e.range.contains(&idx) {
            print!("\x1B[4m");
        }
        print!("{c}");
        if e.range.contains(&idx) {
            print!("\x1B[0m");
        }
    }
    println!();
}
