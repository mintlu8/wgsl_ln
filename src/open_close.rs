use proc_macro2::Delimiter;

pub fn open(d: Delimiter) -> char {
    match d {
        Delimiter::Parenthesis => '(',
        Delimiter::Brace => '{',
        Delimiter::Bracket => '[',
        Delimiter::None => ' ',
    }
}

pub fn close(d: Delimiter) -> char {
    match d {
        Delimiter::Parenthesis => ')',
        Delimiter::Brace => '}',
        Delimiter::Bracket => ']',
        Delimiter::None => ' ',
    }
}
