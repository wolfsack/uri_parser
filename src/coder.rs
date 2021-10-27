use crate::err::Error;

use crate::statics;

use std::collections::{HashSet, VecDeque};

#[cfg(test)]
use super::TestCase;

#[derive(Debug)]
pub struct Decoder {
    input: VecDeque<char>,
    output: Vec<char>,
    viable_chars: &'static HashSet<char>,
    finished: bool,
}

impl Decoder {
    pub fn new(input: Vec<char>, viable_chars: &'static HashSet<char>) -> Self {
        Decoder {
            input: VecDeque::from(input),
            output: Vec::<char>::new(),
            viable_chars,
            finished: false,
        }
    }

    pub fn decode(&mut self) -> Result<String, Error> {
        if self.finished {
            return Ok(self.output.iter().collect());
        }
        while !self.input.is_empty() {
            let char = match self.input.pop_front() {
                None => unreachable!(),
                Some(c) => c,
            };

            // if a percent endcoded character was found
            if char == '%' {
                let (hex_char_1, hex_char_2) =
                        // get next two characters
                        match (self.input.pop_front(), self.input.pop_front()) {
                            // if there are two character
                            (Some(h1), Some(h2)) => {
                                // both character have to be hex values
                                if statics::HEXDIG.contains(&h1) && statics::HEXDIG.contains(&h1) {
                                    (h1, h2)
                                } else {
                                    return Err(Error::IllegalPercentEncoding);
                                }
                            },
                            // if there are not two character
                            (_, _) => return Err(Error::IllegalPercentEncoding),
                        };

                // transform two hex character into ints
                let (int1, int2) = match (hex_char_1.to_digit(16), hex_char_2.to_digit(16)) {
                    // we already made sure its a hex char, 
                    // and translating from hex to dec is guaranteed to be smaller then u8 max
                    #[allow(clippy::cast_possible_truncation)]
                    (Some(i1), Some(i2)) => (i1 as u8, i2 as u8),
                    (_, _) => return Err(Error::IllegalPercentEncoding),
                };
                // first hex value can max. 7 so char is in ASCII range
                if int1 > 7 {
                    return Err(Error::IllegalPercentEncoding);
                };

                // transform two ints into one char
                // max would be 7F -> 7 and 15 ->  127
                let decoded_char = (int1 * 16 + int2) as char;
                self.output.push(decoded_char);
            }
            // check if the found character is allowed
            else if self.viable_chars.contains(&char) {
                // if allowed push it onto output
                self.output.push(char);
            }
            // character is not allowed
            else {
                return Err(Error::IllegalCharacter);
            }
        }

        self.finished = true;
        // return output as string
        Ok(self.output.iter().collect())
    }

}


#[derive(Debug)]
pub struct Encoder {
    input: VecDeque<char>,
    output: Vec<char>,
    viable_chars: &'static HashSet<char>,
    finished: bool,
}

impl Encoder {

    pub fn new(input: Vec<char>, viable_chars: &'static HashSet<char>) -> Self {
        Encoder {
            input: VecDeque::from(input),
            output: Vec::<char>::new(),
            viable_chars,
            finished: false,
        }
    }

    pub fn encode(&mut self) -> Result<String, Error> {
        if self.finished {
            return Ok(self.output.iter().collect());
        }
        while !self.input.is_empty() {
            let char = match self.input.pop_front() {
                None => unreachable!(),
                Some(c) => c,
            };
            

            if self.viable_chars.contains(&char) {
                self.output.push(char);
            } 
            // if the character is not allowed try to encode it
            else {

                let dec = char as u8;

                // check character is a ASCII character
                if dec > 127 {
                    return Err(Error::IllegalCharacter);
                }

                let x:Vec<char> = format!("{:2X}", dec).chars().collect();
                self.output.push('%');
                self.output.push(match x.first(){
                    // None case should be unreachable
                    None => return Err(Error::IllegalPercentEncoding),
                    Some(c) =>  *c
                });
                self.output.push(match x.last(){
                    // None case should be unreachable
                    None => return Err(Error::IllegalPercentEncoding),
                    Some(c) =>  *c
                });
            }

        }

        self.finished = true;
        // return output as string
        Ok(self.output.iter().collect())
    }

}

#[test]
fn encoder_decode_ok() {
    let tests = [
        TestCase {
            case: {
                let chars: Vec<char> = "A".chars().into_iter().collect();
                let mut encoder = Encoder::new(chars, &statics::ALPHA);
                encoder.encode()

            },
            expected: Ok(String::from("A")),
        },
        TestCase {
            case: {
                let chars: Vec<char> = "Hello World!".chars().into_iter().collect();
                let mut encoder = Encoder::new(chars, &statics::ALPHA);
                encoder.encode()

            },
            expected: Ok(String::from("Hello%20World%21")),
        },
        TestCase {
            case: {
                let chars: Vec<char> = " ".chars().into_iter().collect();
                let mut encoder = Encoder::new(chars, &statics::ALPHA);
                encoder.encode()

            },
            expected: Ok(String::from("%20")),
        },
        TestCase {
            case: {
                let chars: Vec<char> = "-".chars().into_iter().collect();
                let mut encoder = Encoder::new(chars, &statics::ALPHA);
                encoder.encode()

            },
            expected: Ok(String::from("%2D")),
        },
        TestCase {
            case: {
                let chars: Vec<char> = "%".chars().into_iter().collect();
                let mut encoder = Encoder::new(chars, &statics::ALPHA);
                encoder.encode()

            },
            expected: Ok(String::from("%25")),
        },
        TestCase {
            case: {
                let chars: Vec<char> = "ABCDEF".chars().into_iter().collect();
                let mut encoder = Encoder::new(chars, &statics::DIGIT);
                encoder.encode()

            },
            expected: Ok(String::from("%41%42%43%44%45%46")),
        },
        TestCase {
            case: {
                let chars: Vec<char> = "12345".chars().into_iter().collect();
                let mut encoder = Encoder::new(chars, &statics::ALPHA);
                encoder.encode()

            },
            expected: Ok(String::from("%31%32%33%34%35")),
        },
        TestCase {
            case: {
                let chars: Vec<char> = {
                    let mut vec:Vec<char> = Vec::new();
                    vec.push('\u{7F}');
                    vec
                };
                let mut encoder = Encoder::new(chars, &statics::ALPHA);
                encoder.encode()

            },
            expected: Ok(String::from("%7F")),
        },
    ];

    for test in tests.iter() {
        assert_eq!(test.case, test.expected)
    }
}

#[test]
fn encoder_decode_err() {
    let tests = [
        TestCase {
            case: {
                let chars: Vec<char> = {
                    let mut vec:Vec<char> = Vec::new();
                    vec.push('\u{81}');
                    vec
                };
                let mut encoder = Encoder::new(chars, &statics::ALPHA);
                encoder.encode()

            },
            expected: Err(Error::IllegalCharacter),
        },
        
    ];

    for test in tests.iter() {
        assert_eq!(test.case, test.expected)
    }
}


#[test]
fn decoder_decode_ok() {
    let tests = [
        TestCase {
            case: {
                let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                    .chars()
                    .into_iter()
                    .collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().unwrap()
            },
            expected: String::from("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        },
        TestCase {
            case: {
                let chars = "1234567890".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::DIGIT);
                decoder.decode().unwrap()
            },
            expected: String::from("1234567890"),
        },
        TestCase {
            case: {
                let chars = ":/?#[]@".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::GEN_DELIMS);
                decoder.decode().unwrap()
            },
            expected: String::from(":/?#[]@"),
        },
        TestCase {
            case: {
                let chars = "!$&\'()*+,;='".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::SUB_DELIMS);
                decoder.decode().unwrap()
            },
            expected: String::from("!$&\'()*+,;='"),
        },
        TestCase {
            case: {
                let chars = "%20".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().unwrap()
            },
            expected: String::from(" "),
        },
        TestCase {
            case: {
                let chars = "%2B%2D".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::DIGIT);
                decoder.decode().unwrap()
            },
            expected: String::from("+-"),
        },
        TestCase {
            case: {
                let chars = "%0A".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::GEN_DELIMS);
                decoder.decode().unwrap()
            },
            expected: String::from("\n"),
        },
        TestCase {
            case: {
                let chars = "Hello%20World".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().unwrap()
            },
            expected: String::from("Hello World"),
        },
        TestCase {
            case: {
                let chars = "Hello%20World%21".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().unwrap()
            },
            expected: String::from("Hello World!"),
        },
        TestCase {
            case: {
                let chars = "Hello%0AWorld%21".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().unwrap()
            },
            expected: String::from("Hello\nWorld!"),
        },
        // TODO: How do i handle these cases?
        TestCase {
            case: {
                let chars = "%00".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::GEN_DELIMS);
                decoder.decode().unwrap()
            },
            expected: String::from("\u{0}"),
        },
        TestCase {
            case: {
                let chars = "%7F".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::GEN_DELIMS);
                decoder.decode().unwrap()
            },
            expected: String::from("\u{7f}"),
        },
    ];
    for test in tests.iter() {
        assert_eq!(test.case, test.expected);
    }
}

#[test]
fn decoder_decode_err() {
    let tests = [
        TestCase {
            case: {
                let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!"
                    .chars()
                    .into_iter()
                    .collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalCharacter,
        },
        TestCase {
            case: {
                let chars = "1234567890!".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::DIGIT);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalCharacter,
        },
        TestCase {
            case: {
                let chars = ":/?#[]@!".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::GEN_DELIMS);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalCharacter,
        },
        TestCase {
            case: {
                let chars = "!$&\'()*+,;='A".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::SUB_DELIMS);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalCharacter,
        },
        TestCase {
            case: {
                let chars = "%8F".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalPercentEncoding,
        },
        TestCase {
            case: {
                let chars = "%A".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalPercentEncoding,
        },
        TestCase {
            case: {
                let chars = "%8%20".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalPercentEncoding,
        },
        TestCase {
            case: {
                let chars = "%".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalPercentEncoding,
        },
        TestCase {
            case: {
                let chars = "Hello%2".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalPercentEncoding,
        },
        TestCase {
            case: {
                let chars = "Hello%20World!".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalCharacter,
        },
        TestCase {
            case: {
                let chars = "Hello%AWorld%21".chars().into_iter().collect();
                let mut decoder = Decoder::new(chars, &statics::ALPHA);
                decoder.decode().err().unwrap()
            },
            expected: Error::IllegalPercentEncoding,
        },
        
    ];
    for test in tests.iter() {
        assert_eq!(test.case, test.expected);
    }
}
