use std::collections::VecDeque;

use crate::statics;

struct IPv6Parser {
    had_double_colon: bool,
    input: VecDeque<char>,
    colon_counter: u8,
    char_counter: u8,
    max_colons: u8,
}

impl IPv6Parser {
    fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().into_iter().collect();
        IPv6Parser {
            had_double_colon: false,
            input: VecDeque::from(chars),
            colon_counter: 0,
            char_counter: 0,
            max_colons: 7,
        }
    }

    fn is_valid(&mut self) -> bool {
        if self.input.is_empty() {
            return false;
        }

        let mut colon = false;
        while !self.input.is_empty() {
            let char = match self.input.pop_front() {
                Some(c) => c,
                None => return false,
            };

            if char == ':' {
                // max number of colons was already reached
                if self.colon_counter >= self.max_colons {
                    return false;
                }

                // if the last character was a ":"
                if colon {
                    // double colon multiple times
                    if self.had_double_colon {
                        return false;
                    }
                    self.had_double_colon = true;
                    self.colon_counter += 1;
                } else {
                    colon = true;
                    self.colon_counter += 1;
                    self.char_counter = 0;
                }
            }
            // character is not a hexdigit
            else {
                // max 4 hexdigits in one segment
                if self.char_counter > 4 {
                    return false;
                }
                if !statics::HEXDIG.contains(&char) {
                    return false;
                }
                colon = false;
                self.char_counter += 1;
            }
        }

        // check if ip is too short
        self.max_colons <= self.colon_counter || self.had_double_colon
        
    }
}

//  ###########################

pub fn is_valid_ip_v_future(input: &str) -> bool {
    // "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
    if input.starts_with('v') || input.starts_with('V') {
        let mut chars = input.chars();
        // get second and third character
        // warning: chars.nth() uses next()
        let (hexdig, dot) = match (chars.nth(1), chars.next()) {
            (Some(c), Some(d)) => (c, d),
            (_, _) => return false,
        };
        // if second character is a hexdigit and third character a "." and contains no percent encoded character
        if statics::HEXDIG.contains(&hexdig) && dot == '.' && !input.contains('%') {
            // check if every char is valid
            for char in input[3..].chars() {
                if !statics::STRIPPED_IP_FUTURE.contains(&char) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
    // doesn't start with a "v"
    else {
        false
    }
}

#[test]
fn is_valid_ip_v_future_test() {
    assert_eq!(is_valid_ip_v_future("v7.abc"), true);
    assert_eq!(is_valid_ip_v_future("VF.127.0.0.1"), true);
    assert_eq!(is_valid_ip_v_future("x7.abc"), false);
    assert_eq!(is_valid_ip_v_future("xX.abc"), false);
    assert_eq!(is_valid_ip_v_future("V7:127.0.0.1"), false);
}

//  ###########################

pub fn is_valid_ip_v6(input: &str) -> bool {
    let mut parser = IPv6Parser::new(input);
    parser.is_valid()
}

#[test]
fn is_valid_ip_v6_test() {
    assert_eq!(
        is_valid_ip_v6("2001:db8:3333:4444:5555:6666:7777:8888"),
        true
    );
    assert_eq!(
        is_valid_ip_v6("2001:db8:3333:AAAA:BBBB:CCCC:DDDD:EEEE"),
        true
    );
    assert_eq!(is_valid_ip_v6("2001:db8:3333::BBBB:CCCC:DDDD:EEEE"), true);
    assert_eq!(is_valid_ip_v6("200:db8:333::BBB:CCC:DDD:EEE"), true);
    assert_eq!(is_valid_ip_v6("::"), true);
    assert_eq!(is_valid_ip_v6("::FFFF"), true);
    assert_eq!(is_valid_ip_v6("2001:db8:3333:AAAA:BBBB::"), true);
    assert_eq!(is_valid_ip_v6("2001:db8:3333:BBBB:CCCC:DDDD"), false);
    assert_eq!(is_valid_ip_v6("2001:db8:3333:BBBB:CCCC:DDDD:"), false);
    assert_eq!(is_valid_ip_v6("2001:db8:3333::BBBBB:CCCC::"), false);
    assert_eq!(is_valid_ip_v6("2001:db8:3333::BBBB:CCCC::"), false);
    assert_eq!(is_valid_ip_v6("200:db8:333::BBB:CCC:DDD:GGGG"), false);
    assert_eq!(is_valid_ip_v6("200:db8:333:::BBB:CCC:DDD"), false);
    assert_eq!(is_valid_ip_v6("200:db8:333:AAA:BBB:CCC:DDD:EEE:FFF"), false);
    assert_eq!(is_valid_ip_v6("200:db8:333::AAA:BBB:CCC:DDD:EEE"), false);
    assert_eq!(is_valid_ip_v6("200:db8:333:AAA:BBB:CCC:DDD:EEE::"), false);
}

//  ###########################
