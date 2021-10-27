use std::collections::HashSet;

// ALPHA contains all allowed letters
lazy_static! {
    pub static ref ALPHA: HashSet<char> = ('a'..='z')
        .into_iter()
        .chain(('A'..='Z').into_iter())
        .collect::<HashSet<char>>();
}

// Digit contains all allowed digits
lazy_static! {
    pub static ref DIGIT: HashSet<char> = ('0'..='9').into_iter().collect::<HashSet<char>>();
}

// HEXDIG contains all allowed hex chars
lazy_static! {
    pub static ref HEXDIG: HashSet<char> = ('0'..='9')
        .into_iter()
        .chain(('a'..='f').into_iter())
        .chain(('A'..='F').into_iter())
        .collect::<HashSet<char>>();
}

// GEN_DELIMS
lazy_static! {
    pub static ref GEN_DELIMS: HashSet<char> = [':', '/', '?', '#', '[', ']', '@']
        .iter()
        .copied()
        .collect::<HashSet<char>>();
}

// SUB_DELIMS
lazy_static! {
    pub static ref SUB_DELIMS: HashSet<char> =
        ['!', '$', '&', '\'', '(', ')', '*', '+', ',', ';', '=',]
            .iter()
            .copied()
            .collect::<HashSet<char>>();
}

// RESERVED containers all characters that are reserved/ hold special syntactic meanings
lazy_static! {
    pub static ref RESERVED: HashSet<char> = SUB_DELIMS
        .iter()
        .chain(GEN_DELIMS.iter())
        .copied()
        .collect::<HashSet<char>>();
}

// UNRESERVED containers all characters that can be freely used
lazy_static! {
    pub static ref UNRESERVED: HashSet<char> = ALPHA
        .iter()
        .chain(DIGIT.iter())
        .chain(['-', '.', '_', '~'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// SCHEME contains all characters that can be used in scheme
lazy_static! {
    pub static ref SCHEME: HashSet<char> = ALPHA
        .iter()
        .chain(DIGIT.iter())
        .chain(['+', '-', '.'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// USER_INFO contains all characters that can be used in userinfo
// UNRESERVED / SUB_DELIMS
lazy_static! {
    pub static ref USER_INFO: HashSet<char> = UNRESERVED
        .iter()
        .chain(SUB_DELIMS.iter())
        .copied()
        .collect::<HashSet<char>>();
}

// STRIPPED_IP_FUTURE contains all characters that can be used in STRIPPED_IP_FUTURE
// unreserved / sub-delims / ":" 
lazy_static! {
    pub static ref STRIPPED_IP_FUTURE: HashSet<char> = UNRESERVED
        .iter()
        .chain(SUB_DELIMS.iter())
        .chain([':'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// IP_6 contains all characters that can be used in IP_6
// unreserved / sub-delims / ":" 
lazy_static! {
    pub static ref IP_6: HashSet<char> = HEXDIG
        .iter()
        .chain([':'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// REG_NAME contains all characters that can be used in REG_NAME
// unreserved / pct-encoded / sub-delims / "."
lazy_static! {
    pub static ref REG_NAME: HashSet<char> = UNRESERVED
        .iter()
        .chain(SUB_DELIMS.iter())
        .chain(['.'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// PATH contains all characters that can be used in path
// UNRESERVED / SUB_DELIMS / ":" / "@" / "/"
lazy_static! {
    pub static ref PATH: HashSet<char> = UNRESERVED
        .iter()
        .chain(SUB_DELIMS.iter())
        .chain([':', '@', '/'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// QUERY contains all characters that can be used in query
// UNRESERVED / SUB_DELIMS / ":" / "@" / "/" / "?"
lazy_static! {
    pub static ref QUERY: HashSet<char> = UNRESERVED
        .iter()
        .chain(SUB_DELIMS.iter())
        .chain([':', '@', '/', '?'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// FRAGMENT contains all characters that can be used in fragment
// UNRESERVED / SUB_DELIMS / ":" / "@" / "/" / "?"
lazy_static! {
    pub static ref FRAGMENT: HashSet<char> = UNRESERVED
        .iter()
        .chain(SUB_DELIMS.iter())
        .chain([':', '@', '/', '?'].iter())
        .copied()
        .collect::<HashSet<char>>();
}
