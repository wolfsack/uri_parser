use crate::{coder::{Decoder, Encoder}, err::Error, ip, statics};

#[derive(Debug)]
pub struct Authority {
    pub userinfo: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
}

impl PartialEq for Authority {
    fn eq(&self, other: &Self) -> bool {
        self.userinfo == other.userinfo && self.host == other.host && self.port == other.port
    }
}

impl Authority {
    /// # Errors
    ///
    /// Will return 'Error' if given string contains characters that are not valid in their respctive parts.
    pub fn parse(auth_string: &str) -> Result<Option<Self>, Error> {
        if auth_string.is_empty() {
            return Ok(None);
        }

        let (userinfo, rest): (Option<&str>, Option<&str>) = Self::split_userinfo(auth_string);

        let (host, port): (Option<&str>, Option<&str>) = match rest {
            None => (None, None),
            Some(rest) => Self::split_host(rest)?,
        };

        let parsed_userinfo: Option<String> = match userinfo {
            None => None,
            Some(useri) => Some(Self::parse_userinfo(useri)?),
        };

        let parsed_host: Option<String> = match host {
            None => None,
            Some(h) => Some(Self::parse_host(h)?),
        };

        let parsed_port: Option<u16> = match port {
            None => None,
            Some(p) => Some(Self::parse_port(p)?),
        };

        match (&parsed_userinfo, &parsed_host, &parsed_port) {
            (None, None, None) => Ok(None),
            (_, _, _) => Ok(Some(Authority {
                userinfo: parsed_userinfo,
                host: parsed_host,
                port: parsed_port,
            })),
        }
    }

    fn split_userinfo(auth_string: &str) -> (Option<&str>, Option<&str>) {
        match auth_string.split_once('@') {
            None => (None, Some(auth_string)),
            Some((useri, rest)) => {
                match (useri, rest) {
                    // "@"
                    ("", "") => (None, None),

                    // "@example.com"
                    ("", r) => (None, Some(r)),

                    // "user1@"
                    (i, "") => (Some(i), None),

                    // "user1@example.com"
                    (i, r) => (Some(i), Some(r)),
                }
            }
        }
    }

    fn split_host(host_port: &str) -> Result<(Option<&str>, Option<&str>), Error> {
        // if host is a IP-literal make sure i look for a port after the ip-address is closed
        let delim = if host_port.starts_with('[') {
            match host_port.find(']') {
                None => return Err(Error::IllegaHostDefinition),
                Some(i) => i + 1,
            }
        } 
        // search the whole input for ":"
        else {
            0
        };

        match &host_port[delim..].find(':') {
            // no port found
            None => Ok((Some(host_port), None)),
            Some(colon) => {
                // split at the original inputs position where the found ":" is
                let parts = host_port.split_at(delim + colon);

                match (parts.0, &parts.1[1..]) {
                    // ":"
                    ("", "") => Ok((None, None)),

                    // ":8080"
                    ("", p) => Ok((None, Some(p))),

                    // "example.com:"
                    (h, "") => Ok((Some(h), None)),

                    // "example.com:8080"
                    (h, p) => Ok((Some(h), Some(p))),
                }
            }
        }
    }

    fn parse_userinfo(user_info: &str) -> Result<String, Error> {
        // build decoder
        let chars: Vec<char> = user_info.chars().collect();
        let mut userinfo_decoder = Decoder::new(chars, &statics::USER_INFO);

        // decode input and remap to a more discriptive error
        match userinfo_decoder.decode() {
            Err(err) => match err {
                Error::IllegalCharacter => Err(Error::UserinfoIllegalCharacter),
                _ => Err(err),
            },
            Ok(str) => Ok(str),
        }
    }

    fn parse_host(host: &str) -> Result<String, Error> {
        // check what kind of host is given
        let starts_with = host.starts_with('[');
        let ends_with = host.starts_with('[');

        // if a IP-literal is given it needs to start with "[" and ends with "]"
        if (starts_with && !ends_with) || (!starts_with && ends_with) {
            // if host doesnt start with "[" but doesn't ends with "]" or
            // doesn't start with "[" but ends with "]"
            return Err(Error::IllegaHostDefinition);
        }
        //  if host starts with "[" and ends with "]"
        else if starts_with && ends_with {
            //  [::]  is the minimal length
            if host.len() < 4 {
                return Err(Error::IllegaHostDefinition);
            }

            let host_stripped = &host[1..host.len() - 2];

            // IPvFuture
            if host_stripped.starts_with('v') {
                if ip::is_valid_ip_v_future(host_stripped) {
                    return Ok(String::from(host));
                } 
                return Err(Error::IllegalIPvFuture);
            }
            // IPv6 address
            if ip::is_valid_ip_v6(host_stripped) {
                return Ok(String::from(host));
            } 
            return Err(Error::IllegalIPv6);
            
        }
        // if its not a IP-literal
        //
        // RFC 3986 January 2005 3.2.2. Host
        // The syntax rule for host is ambiguous because it does not completely
        // distinguish between an IPv4address and a reg-name.
        
        let chars: Vec<char> = host.chars().into_iter().collect();
        let mut decoder = Decoder::new(chars, &statics::REG_NAME);
        match decoder.decode() {
            Ok(result) => Ok(result),
            Err(err) => {
                match err {
                    Error::IllegalCharacter => Err(Error::HostIllegalCharacter),
                    _ => Err(err),
                }
            }
        }
        
    }

    fn parse_port(port_str: &str) -> Result<u16, Error> {
        match port_str.parse::<u16>() {
            Err(_) => Err(Error::ParsePortError),
            Ok(port) => Ok(port),
        }
    }

    /// # Errors
    /// 
    /// Can return Errors if the Authority parts contain characters that are not ASCII characters.
    /// This can happen when building an Authority without the provided functions
    pub fn stringify(& self) -> Result<Option<String>, Error> {
        let mut output = String::new();
        let mut encoder:Encoder;

        if self.port.is_none() && self.host.is_none() && self.userinfo .is_none() {
            return Ok(None);
        };

        if let Some(ui) = &self.userinfo {
            let chars:Vec<char> = ui.chars().into_iter().collect();
            encoder = Encoder::new(chars, &statics::USER_INFO);
            output.push_str(&encoder.encode()?);
            output.push('@');
        };

        if let Some(ho) = &self.host {
            if ho.starts_with('[') {
                output.push_str(ho);
            } else {
                let chars:Vec<char> = ho.chars().into_iter().collect();
                encoder = Encoder::new(chars, &statics::REG_NAME);
                output.push_str(&encoder.encode()?);
            }
        };

        if let Some(po) = self.port {
            output.push(':');
            output.push_str(&po.to_string());
        };

        Ok(Some(output))
    }

    #[must_use]
    pub fn userinfo(&self) -> Option<&str> {
        self.userinfo.as_deref()
    }

    #[must_use]
    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    #[must_use]
    pub fn port(&self) -> Option<u16> {
        self.port
    }
}

#[cfg(test)]
mod tests {

    use crate::{Error, TestCase};
    use super::Authority;

    #[test]
    fn parse_ok() {
        let tests = [
            TestCase {
                case: Authority::parse("example.com").unwrap(),
                expected: Some(Authority {
                    userinfo: None,
                    host: Some(String::from("example.com")),
                    port: None,
                }),
            },
            TestCase {
                case: Authority::parse("user@example.com").unwrap(),
                expected: Some(Authority {
                    userinfo: Some(String::from("user")),
                    host: Some(String::from("example.com")),
                    port: None,
                }),
            },
            TestCase {
                case: Authority::parse("user@example.com:8080").unwrap(),
                expected: Some(Authority {
                    userinfo: Some(String::from("user")),
                    host: Some(String::from("example.com")),
                    port: Some(8080),
                }),
            },
            TestCase {
                case: Authority::parse("example.com:8080").unwrap(),
                expected: Some(Authority {
                    userinfo: None,
                    host: Some(String::from("example.com")),
                    port: Some(8080),
                }),
            },
            TestCase {
                case: Authority::parse("@example.com:8080").unwrap(),
                expected: Some(Authority {
                    userinfo: None,
                    host: Some(String::from("example.com")),
                    port: Some(8080),
                }),
            },
            TestCase {
                case: Authority::parse("example.com:").unwrap(),
                expected: Some(Authority {
                    userinfo: None,
                    host: Some(String::from("example.com")),
                    port: None,
                }),
            },
            TestCase {
                case: Authority::parse("user@:8080").unwrap(),
                expected: Some(Authority {
                    userinfo: Some(String::from("user")),
                    host: None,
                    port: Some(8080),
                }),
            },
        ];

        for test in tests.iter() {
            assert_eq!(test.case, test.expected)
        }
    }

    #[test]
    fn parse_special_characters_ok() {
        let tests = [
            TestCase {
                case: Authority::parse("[2001:db8:3333:4444:5555:6666:7777:8888]").unwrap(),
                expected: Some(Authority {
                    userinfo: None,
                    host: Some(String::from("[2001:db8:3333:4444:5555:6666:7777:8888]")),
                    port: None,
                }),
            },
            TestCase {
                case: Authority::parse("user@[2001:db8:3333::5555:6666:7777:8888]:8080").unwrap(),
                expected: Some(Authority {
                    userinfo: Some(String::from("user")),
                    host: Some(String::from("[2001:db8:3333::5555:6666:7777:8888]")),
                    port: Some(8080),
                }),
            },
            TestCase {
                case: Authority::parse("127.0.0.1").unwrap(),
                expected: Some(Authority {
                    userinfo: None,
                    host: Some(String::from("127.0.0.1")),
                    port: None,
                }),
            },
            TestCase {
                case: Authority::parse("127.0.0.1:8080").unwrap(),
                expected: Some(Authority {
                    userinfo: None,
                    host: Some(String::from("127.0.0.1")),
                    port: Some(8080),
                }),
            },
            TestCase {
                case: Authority::parse("[v7.aaaa:bbbb:cccc::]").unwrap(),
                expected: Some(Authority {
                    userinfo: None,
                    host: Some(String::from("[v7.aaaa:bbbb:cccc::]")),
                    port: None,
                }),
            },
            TestCase {
                case: Authority::parse("[v7.aaaa:bbbb:cccc::]:8080").unwrap(),
                expected: Some(Authority {
                    userinfo: None,
                    host: Some(String::from("[v7.aaaa:bbbb:cccc::]")),
                    port: Some(8080),
                }),
            },
            TestCase {
                case: Authority::parse("user+@example.com+:8080").unwrap(),
                expected: Some(Authority {
                    userinfo: Some(String::from("user+")),
                    host: Some(String::from("example.com+")),
                    port: Some(8080),
                }),
            },
            TestCase {
                case: Authority::parse("www.example().com:").unwrap(),
                expected: Some(Authority {
                    userinfo: None,
                    host: Some(String::from("www.example().com")),
                    port: None,
                }),
            },
            TestCase {
                case: Authority::parse("user=@:8080").unwrap(),
                expected: Some(Authority {
                    userinfo: Some(String::from("user=")),
                    host: None,
                    port: Some(8080),
                }),
            },
        ];

        for test in tests.iter() {
            assert_eq!(test.case, test.expected)
        }
    }

    #[test]
    fn parse_pe_characters_ok() {
        let tests = [
            TestCase {
                case: Authority::parse("u%73er@[2001:db8:3333:4444:5555:6666:7777:8888]").unwrap(),
                expected: Some(Authority {
                    userinfo: Some(String::from("user")),
                    host: Some(String::from("[2001:db8:3333:4444:5555:6666:7777:8888]")),
                    port: None,
                }),
            },
            TestCase {
                case: Authority::parse("user%23@example.com%3F:8080").unwrap(),
                expected: Some(Authority {
                    userinfo: Some(String::from("user#")),
                    host: Some(String::from("example.com?")),
                    port: Some(8080),
                }),
            },
        ];

        for test in tests.iter() {
            assert_eq!(test.case, test.expected)
        }
    }

    #[test]
    fn parse_err() {
        let tests = [
            TestCase {
                case: Authority::parse("user:@[2001:db8:3333:4444:5555:6666:7777:8888]").err().unwrap(),
                expected: Error::UserinfoIllegalCharacter,
            },
            TestCase {
                case: Authority::parse("user#@example.com:8080").err().unwrap(),
                expected: Error::UserinfoIllegalCharacter,
            },
            TestCase {
                case: Authority::parse("user@example.com?:8080").err().unwrap(),
                expected: Error::HostIllegalCharacter,
            },
            TestCase {
                case: Authority::parse("user@example.com:80%50").err().unwrap(),
                expected: Error::ParsePortError,
            },
            TestCase {
                case: Authority::parse("[2001:db8:3333:4444:5555:6666:7777::8888]").err().unwrap(),
                expected: Error::IllegalIPv6,
            },
            TestCase {
                case: Authority::parse("[2001:db8:3333:4444:5555:6666:7777]").err().unwrap(),
                expected: Error::IllegalIPv6,
            },
            TestCase {
                case: Authority::parse("[vX.::]").err().unwrap(),
                expected: Error::IllegalIPvFuture,
            },

        ];

        for test in tests.iter() {
            assert_eq!(test.case, test.expected)
        }
    }

}
