use crate::coder::{Decoder, Encoder};
use crate::statics;
use crate::err::Error;
use crate::Authority;

#[cfg(test)]
use crate::TestCase;

#[derive(Debug)]
pub struct Uri {
    scheme: Option<String>,
    authority: Option<Authority>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl PartialEq for Uri {
    fn eq(&self, other: &Self) -> bool {
        self.scheme == other.scheme
            && self.authority == other.authority
            && self.path == other.path
            && self.query == other.query
            && self.fragment == other.fragment
    }
}

impl Uri {
    /// # Errors
    ///
    /// Will return 'Error' if given string is not a valid URI.
    /// Given URI should comply with RFC3986.
    pub fn parse(uri_string: &str) -> Result<Uri, Error> {

        // "" is a valid "relative reference" URI
        if uri_string.is_empty() {return Ok(Uri{
            scheme: None,
            authority: None,
            path: String::from(""),
            query: None,
            fragment: None
        })}

        let (scheme, without_scheme) = Self::split_scheme(uri_string)?;

        let (fragment, authority_path_query) = Self::split_fragment(without_scheme);

        let (query, authority_path) = Self::split_query(authority_path_query);
        let (authority, path) = Self::split_authority_and_path(authority_path)?;

        let parsed_scheme = match scheme {
            None => None,
            Some(scheme_string) => Some(Self::parse_scheme(scheme_string)?)
        };

        let parsed_fragment = match fragment {
            None => None,
            Some(fragment_string) => Some(Self::parse_fragment(fragment_string)?)
        };

        let parsed_query = match query {
            None => None,
            Some(query_string) => Some(Self::parse_query(query_string)?)
        };

        let parsed_path = Self::parse_path(path)?;


        let parsed_authority = match authority {
            None => None,
            Some(auth_string) => Authority::parse(auth_string)?
        };

        Ok(Uri {
           scheme: parsed_scheme,
           authority: parsed_authority,
           path: parsed_path,
           query: parsed_query,
           fragment: parsed_fragment
        })
    }

    #[must_use]
    pub fn scheme(&self) -> Option<&str> {
        match &self.scheme {
            None => None,
            Some(scheme)  => Some(scheme)
        }
    }
    
    #[must_use]
    pub fn userinfo(&self) -> Option<&str> {
        match &self.authority {
            Some(auth) => auth.userinfo(),
            None => None
        }
    }
    
    #[must_use]
    pub fn host(&self) -> Option<&str>{
        match &self.authority {
            Some(auth) => auth.host(),
            None => None
        }
    }
    
    #[must_use]
    pub fn port(&self) -> Option<u16> {
            match &self.authority {
                Some(auth) => auth.port(),
                None => None
            }
        }
    
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    
    #[must_use]
    pub fn query(&self) -> Option<&str> {
        match &self.query {
            Some(query) => Some(query),
            None => None
        }
    }

    /// # Errors
    /// 
    /// Can return Errors if the Authority parts contain characters that are not ASCII characters.
    /// This can happen when building an Authority without the provided functions
    pub fn stringify(& self) -> Result<String, Error> {
        let mut output = String::new();
        let mut encoder:Encoder;


        if let Some(sch) = &self.scheme {
            let chars:Vec<char> = sch.chars().into_iter().collect();
            encoder = Encoder::new(chars, &statics::SCHEME);
            output.push_str(&encoder.encode()?);
            output.push(':');
        };

        if let Some(au) = &self.authority {
            if let Some(au_str) = au.stringify()? {
                output.push_str("//");
                output.push_str(&au_str);
            }
        };

        let chars:Vec<char> = self.path.chars().into_iter().collect();
        encoder = Encoder::new(chars, &statics::PATH);
        output.push_str(&encoder.encode()?);

        if let Some(qu) = &self.query {
            let chars:Vec<char> = qu.chars().into_iter().collect();
            encoder = Encoder::new(chars, &statics::QUERY);
            output.push('?');
            output.push_str(&encoder.encode()?);
        };

        if let Some(fr) = &self.fragment {
            let chars:Vec<char> = fr.chars().into_iter().collect();
            encoder = Encoder::new(chars, &statics::FRAGMENT);
            output.push('#');
            output.push_str(&encoder.encode()?);
        };

        Ok(output)
    }

    fn split_scheme(uri_string: &str) -> Result<(Option<&str>, &str), Error> {

        let delim  = uri_string.find('/').unwrap_or_else(|| uri_string.len());
        
        uri_string[..delim].find(':').map_or(Ok((None, uri_string)), |scheme_end| 
            if uri_string[..scheme_end].is_empty() {
                 Err(Error::EmptyScheme)
            } else {
                Ok((Some(&uri_string[..scheme_end]), &uri_string[scheme_end+1..]))
            }
        )
    }

    fn split_fragment(without_scheme: &str) ->(Option<&str>, &str) {
        match without_scheme.rsplit_once("#") {
            Some((rest, fragment)) => (Some(fragment), rest),
            None => (None, without_scheme),
        }
    }

    fn split_query(authority_path_query: &str) -> (Option<&str>, &str) {
        if authority_path_query.contains('#') { eprintln!("Warning, Fragment is not removed before possible query is parsed!") }
        match authority_path_query.split_once('?') {
            Some((rest, query_string)) => {
                (Some(query_string), rest)
            }
            None => (None, authority_path_query),
        }
    }

    fn split_authority_and_path(
        authority_path: &str,
    ) -> Result<(Option<&str>, &str), Error> {
        /*
        //  RFC 3986 January 2005 3.2. Authority
        //  Many URI schemes include a hierarchical element for a naming
        //  authority so that governance of the name space defined by the
        //  remainder of the URI is delegated to that authority (which may, in
        //  turn, delegate it further).
        //
        //  The generic syntax provides a common means for distinguishing
        //  an authority based on a registered name or
        //  server address, along with optional port and user information.
        //
        //  The authority component is preceded by a double slash ("//") and is
        //  terminated by the next slash ("/"), question mark ("?"), or number
        //  sign ("#") character, or by the end of the URI.
        */

        let (auth, path): (Result<Option<&str>, Error>, &str) =
            // try to remove mendatory prefix for Authority
            match authority_path.strip_prefix("//") {
                // prefix doesnt exists -> no Authority, input is path
                None => (Ok(None), authority_path),

                // prefix was removed
                Some(stripped_authority_path) => {
                    // check if authority is "" while indicating its existing with "//"
                    if stripped_authority_path.is_empty() {
                        return Err(Error::EmptyAuthority); 
                    }

                    let auth_end: usize = stripped_authority_path
                        // get position where the Authority ends
                        // end of Authority is indicated by '/'
                        .find('/')
                        // if path is "" return whole length
                        .unwrap_or(stripped_authority_path.len());

                    // check if the Authority is instantly limited by "/"
                    // if it is, that would be an empty Authority
                    if auth_end == 0 {
                        return Err(Error::EmptyAuthority);  
                    }

                    // if Authority limiter is the end
                    if auth_end == stripped_authority_path.len() {
                        // return whole input as Authority and
                        // empty path
                        (Ok(Some(stripped_authority_path)), "")
                    } 
                    // not the whole input is the Authority
                    else {
                        (   
                            // Authority is from start to auth_end
                            Ok(Some(&stripped_authority_path[..auth_end])),
                            // path is from auth_end to end
                            &stripped_authority_path[auth_end..],
                        )
                    }
                }
            };

        match auth {
            Ok(auth_option) => Ok((auth_option, path)),
            Err(err) => Err(err),
        }
    }

    fn parse_scheme(scheme_string: &str) -> Result<String, Error> {
        /*
        //  RFC 3986 January 2005 3.1. Scheme
        //  Each URI begins with a scheme name that refers to a specification for
        //  assigning identifiers within that scheme.

        //  As such, the URI syntax is a federated and extensible naming system wherein each scheme’s
        //  specification may further restrict the syntax and semantics of
        //  identifiers using that scheme.

        //  Scheme names consist of a sequence of characters beginning with a
        //  letter and followed by any combination of letters, digits, plus
        //  ("+"), period ("."), or hyphen ("-").  Although schemes are case-
        //  insensitive, the canonical form is lowercase and documents that
        //  specify schemes must do so with lowercase letters.

        //  An implementation should accept uppercase letters as equivalent to lowercase in scheme
        //  names (e.g., allow "HTTP" as well as "http") for the sake of
        //  robustness but should only produce lowercase scheme names for
        //  consistency.
        */

        // no percent encoding allowed in scheme
        if scheme_string.contains('%') {
            return Err(Error::SchemeIllegalCharacter);
        }

        let chars: Vec<char> = scheme_string.chars().into_iter().collect();

        let first_char = match chars.first() {
            Some(c) => *c,
            None => return Err(Error::EmptyScheme),
        };

        //  check if first character is a letter
        if !statics::ALPHA.contains(&first_char) {
            return Err(Error::SchemeIllegalFirstCharacter);
        };

        // decoder is not doing much (we already checked that no pec are in the scheme, see above)
        // its only checking for invalid chars
        let mut decoder = Decoder::new(chars, &statics::SCHEME);
        match decoder.decode() {
            Err(err) => {
                Err(match err {
                    Error::IllegalCharacter => Error::SchemeIllegalCharacter,
                    _ => err,
                })
            }
            Ok(decoded_scheme) => Ok(decoded_scheme.to_lowercase()),
        }
    }

    fn parse_path(path_string: &str) -> Result<String, Error> {
        /*
        //  RFC 3986 January 2005 3.3. Path
        //  If a URI contains an authority component, then the path component
        //  must either be empty or begin with a slash ("/") character.  If a URI
        //  does not contain an authority component, then the path cannot begin
        //  with two slash characters ("//").  
        // <-   doesn't have to be explizitly checked because the splitting already worked with these rules
        //
        //  In addition, a URI reference (Section 4.1) may be a relative-path reference, 
        //  in which case the first path segment cannot contain a colon (":") character.
        */

        if path_string.starts_with("//") {
            return Err(Error::PathIllegalStart);
        }

        let chars:Vec<char> = path_string.chars().into_iter().collect();
        let mut decoder = Decoder::new(chars, &statics::PATH);
        match decoder.decode() {
            Err(err) => {
                Err(match err {
                    Error::IllegalCharacter => Error::PathIllegalCharacter,
                    _ => err,
                })
            },
            Ok(result) => Ok(result)
        }

    }

    fn parse_query(query_string: &str) -> Result<String, Error> {
        let chars:Vec<char> = query_string.chars().into_iter().collect();
        let mut decoder = Decoder::new(chars, &statics::QUERY);
        match decoder.decode() {
            Err(err) => {
                Err(match err {
                    Error::IllegalCharacter => Error::QueryIllegalCharacter,
                    _ => err,
                })
            }
            Ok(decoded_query) => Ok(decoded_query),
        }
    }

    fn parse_fragment(fragment_string: &str) -> Result<String, Error> {
        let chars:Vec<char> = fragment_string.chars().into_iter().collect();
        let mut decoder = Decoder::new(chars, &statics::FRAGMENT);
        match decoder.decode() {
            Err(err) => {
                Err(match err {
                    Error::IllegalCharacter => Error::FragmentIllegalCharacter,
                    _ => err,
                })
            }
            Ok(decoded_fragment) => Ok(decoded_fragment),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_parse_ok() {
        let tests = [
            TestCase {
                case: Uri::parse("http:").unwrap(),
                expected: Uri {
                    scheme: Some(String::from("http")),
                    authority: None,
                    path: String::from(""),
                    query: None,
                    fragment: None,
                },
            },
            TestCase {
                case: Uri::parse("http://example.com").unwrap(),
                expected: Uri {
                    scheme: Some(String::from("http")),
                    authority: Some(Authority{
                        userinfo: None,
                        host: Some(String::from("example.com")),
                        port:None
                    }),
                    path: String::from(""),
                    query: None,
                    fragment: None,
                },
            },
            TestCase {
                case: Uri::parse("http://user@example.com").unwrap(),
                expected: Uri {
                    scheme: Some(String::from("http")),
                    authority: Some(Authority{
                        userinfo: Some(String::from("user")),
                        host: Some(String::from("example.com")),
                        port:None
                    }),
                    path: String::from(""),
                    query: None,
                    fragment: None,
                },
            },
            TestCase {
                case: Uri::parse("http://user@example.com:8080").unwrap(),
                expected: Uri {
                        scheme: Some(String::from("http")),
                        authority: Some(Authority{
                            userinfo: Some(String::from("user")),
                            host: Some(String::from("example.com")),
                            port: Some(8080)
                        }),
                        path: String::from(""),
                        query: None,
                        fragment: None,
                },
            },
            TestCase {
                case: Uri::parse("http://user@example.com:8080?name=bob").unwrap(),
                expected: Uri {
                    scheme: Some(String::from("http")),
                    authority: Some(Authority{
                        userinfo: Some(String::from("user")),
                        host: Some(String::from("example.com")),
                        port: Some(8080)
                    }),
                    path: String::from(""),
                    query: Some(String::from("name=bob")),
                    fragment: None,
                },
            },
            TestCase {
                case: Uri::parse("http://user@example.com:8080/this/is/a/path?name=bob").unwrap(),
                expected: Uri {
                    scheme: Some(String::from("http")),
                    authority: Some(Authority{
                    userinfo: Some(String::from("user")),
                        host: Some(String::from("example.com")),
                        port: Some(8080)
                        }),
                    path: String::from("/this/is/a/path"),
                    query: Some(String::from("name=bob")),
                    fragment: None,
                },
            },
            TestCase {
                case: Uri::parse("http://user@example.com:8080?name=bob#page3").unwrap(),
                expected: Uri {
                    scheme: Some(String::from("http")),
                    authority: Some(Authority{
                        userinfo: Some(String::from("user")),
                        host: Some(String::from("example.com")),
                        port: Some(8080)
                    }),
                    path: String::from(""),
                    query: Some(String::from("name=bob")),
                    fragment: Some(String::from("page3")),
                },
            },
            TestCase {
                case: Uri::parse(&String::from("urn:oasis:names:specification:docbook:dtd:xml:4.1.2")).unwrap(),
                expected: Uri {
                    scheme: Some(String::from("urn")),
                    authority: None,
                    path: String::from("oasis:names:specification:docbook:dtd:xml:4.1.2"),
                    query: None,
                    fragment: None,
                },
            },
            TestCase {
                case: Uri::parse(&String::from("mailto:John.Doe@example.com")).unwrap(),
                expected: Uri {
                    scheme: Some(String::from("mailto")),
                    authority: None,
                    path: String::from("John.Doe@example.com"),
                    query: None,
                    fragment: None,
                },
            },
            TestCase {
                case: Uri::parse(&String::from("telnet://192.0.2.16:80/")).unwrap(),
                expected: Uri {
                    scheme: Some(String::from("telnet")),
                    authority: Some(Authority{
                        userinfo: None,
                        host: Some(String::from("192.0.2.16")),
                        port: Some(80)
                    }),
                    path: String::from("/"),
                    query: None,
                    fragment: None,
                },
            },
            TestCase {
                case: Uri::parse(&String::from("http://user@[2001:db8:3333::5555:6666:7777:8888]:8080")).unwrap(),
                expected: Uri {
                    scheme: Some(String::from("http")),
                    authority: Some(Authority{
                        userinfo: Some(String::from("user")),
                        host: Some(String::from("[2001:db8:3333::5555:6666:7777:8888]")),
                        port: Some(8080)
                    }),
                    path: String::from(""),
                    query: None,
                    fragment: None,
                },
            },
        ];


        for test in tests.iter() {
            assert_eq!(test.case, test.expected);
        }
    }

    #[test]
    fn uri_parse_err() {
                let tests = [
                    TestCase {
                        case: Uri::parse(&String::from(":")).err().unwrap(),
                        expected: Error::EmptyScheme,
                    },
                    TestCase {
                        case: Uri::parse(&String::from("://example.com")).err().unwrap(),
                        expected: Error::EmptyScheme,
                    },
                    TestCase {
                        case: Uri::parse(&String::from(":/api/v2/test")).err().unwrap(),
                        expected: Error::EmptyScheme,
                    }
                ];

                for test in tests.iter() {
                    assert_eq!(test.case, test.expected);
                }
            }

    #[test]
    fn uri_parse_scheme_ok() {
        let tests = [
            TestCase{
                case: Uri::parse("http:").unwrap().scheme().map(|x| x.to_owned()),
                expected: Some(String::from("http"))
            },
            TestCase{
                case: Uri::parse("http+:").unwrap().scheme().map(|x| x.to_owned()),
                expected: Some(String::from("http+"))
            },
            TestCase{
                case: Uri::parse("http.:").unwrap().scheme().map(|x| x.to_owned()),
                expected: Some(String::from("http."))
            },
            TestCase{
                case: Uri::parse("HttP:").unwrap().scheme().map(|x| x.to_owned()),
                expected: Some(String::from("http"))
            },
            TestCase{
                case: Uri::parse("http://example.com").unwrap().scheme().map(|x| x.to_owned()),
                expected: Some(String::from("http"))
            },
            TestCase{
                case: Uri::parse("http:/this/is/a/path").unwrap().scheme().map(|x| x.to_owned()),
                expected: Some(String::from("http"))
            },
            TestCase{
                case: Uri::parse("//example.com").unwrap().scheme().map(|x| x.to_owned()),
                expected: None
            },
            TestCase{
                case: Uri::parse("/this/is/a/path").unwrap().scheme().map(|x| x.to_owned()),
                expected: None
            },
            TestCase{
                case: Uri::parse("abc/xyz").unwrap().scheme().map(|x| x.to_owned()),
                expected: None
            },
        ];

        for test in tests.iter() {
            assert_eq!(test.case,  test.expected);
        }
    }
            
    #[test]
    fn uri_parse_scheme_err() {
        let tests = [
            TestCase{
                case: Uri::parse(":").unwrap_err(),
                expected: Error::EmptyScheme
            },
            TestCase{
                case: Uri::parse("://example.com").unwrap_err(),
                expected: Error::EmptyScheme
            },
            TestCase{
                case: Uri::parse("1ttp://example.com").unwrap_err(),
                expected: Error::SchemeIllegalFirstCharacter
            },
            TestCase{
                case: Uri::parse("http%70://example.com").unwrap_err(),
                expected: Error::SchemeIllegalCharacter
            },
            TestCase{
                case: Uri::parse("http%70://example.com").unwrap_err(),
                expected: Error::SchemeIllegalCharacter
            },
            TestCase{
                case: Uri::parse("http%70://example.com").unwrap_err(),
                expected: Error::SchemeIllegalCharacter
            },
            TestCase{
                case: Uri::parse("http#://example.com").unwrap_err(),
                expected: Error::SchemeIllegalCharacter
            },
        ];

        for test in tests.iter() {
            assert_eq!(test.case,  test.expected);
        }
    }
  
    #[test]
    fn uri_parse_path_ok() {
        let tests = [
            TestCase{
                case: Uri::parse("http:").unwrap().path().to_owned(),
                expected: String::from("")
            },
            TestCase{
                case: Uri::parse("http://example.com").unwrap().path().to_owned(),
                expected:  String::from("")
            },
            TestCase{
                case: Uri::parse("http://example.com/").unwrap().path().to_owned(),
                expected:  String::from("/")
            },
            TestCase{
                case: Uri::parse("http://example.com/this/is/a/path").unwrap().path().to_owned(),
                expected:  String::from("/this/is/a/path")
            },
            TestCase{
                case: Uri::parse("http://example.com/this/is/a/path/").unwrap().path().to_owned(),
                expected:  String::from("/this/is/a/path/")
            },
            TestCase{
                case: Uri::parse("http://example.com/this/is//path").unwrap().path().to_owned(),
                expected:  String::from("/this/is//path")
            },
            TestCase{
                case: Uri::parse("http:/this/is/a/path").unwrap().path().to_owned(),
                expected:  String::from("/this/is/a/path")
            },
            TestCase{
                case: Uri::parse("//user@:/this/is/a/path").unwrap().path().to_owned(),
                expected:  String::from("/this/is/a/path")
            },
            TestCase{
                case: Uri::parse("http:/this/is/a/(path)").unwrap().path().to_owned(),
                expected:  String::from("/this/is/a/(path)")
            },
            TestCase{
                case: Uri::parse("http:/this/is%20a/(path)").unwrap().path().to_owned(),
                expected:  String::from("/this/is a/(path)")
            },
            TestCase{
                case: Uri::parse("//example.com/this/is/a/path").unwrap().path().to_owned(),
                expected:  String::from("/this/is/a/path")
            },
            TestCase{
                case: Uri::parse("/this/is/a/relativ/path").unwrap().path().to_owned(),
                expected:  String::from("/this/is/a/relativ/path")
            },
            TestCase{
                case: Uri::parse("/").unwrap().path().to_owned(),
                expected:  String::from("/")
            },
            TestCase{
                case: Uri::parse("").unwrap().path().to_owned(),
                expected:  String::from("")
            },
            TestCase{
                case: Uri::parse("//example.com/").unwrap().path().to_owned(),
                expected:  String::from("/")
            },
            TestCase{
                case: Uri::parse("./this:that").unwrap().path().to_owned(),
                expected:  String::from("./this:that")
            },
        ];

        for test in tests.iter() {
            assert_eq!(test.case,  test.expected);
        }
    }

    #[test]
    fn uri_parse_path_err() {
        let tests = [
            TestCase{
                case: Uri::parse("http://example.com//test").unwrap_err(),
                expected: Error::PathIllegalStart
            },
            TestCase{
                case: Uri::parse("//example.com/[test/12").unwrap_err(),
                expected: Error::PathIllegalCharacter
            },
            TestCase{
                case: Uri::parse("//example.com/test]/34").unwrap_err(),
                expected: Error::PathIllegalCharacter
            },
            TestCase{
                case: Uri::parse("//example.com/[test]/56").unwrap_err(),
                expected: Error::PathIllegalCharacter
            },
        ];

        for test in tests.iter() {
            assert_eq!(test.case,  test.expected);
        }

    }

    #[test]
    fn uri_parse_query_ok() {
        let tests = [
            TestCase{
                case: Uri::parse("http:").unwrap().query().map(|x| x.to_owned() ),
                expected: None
            },
            TestCase{
                case: Uri::parse("http://example.com").unwrap().query().map(|x| x.to_owned() ),
                expected: None
            },
            TestCase{
                case: Uri::parse("http://example.com?").unwrap().query().map(|x| x.to_owned() ),
                expected: Some(String::from(""))
            },
            TestCase{
                case: Uri::parse("http://example.com?name").unwrap().query().map(|x| x.to_owned() ),
                expected: Some(String::from("name"))
            },
            TestCase{
                case: Uri::parse("http://example.com?name=").unwrap().query().map(|x| x.to_owned() ),
                expected: Some(String::from("name="))
            },
            TestCase{
                case: Uri::parse("http://example.com?name=bob").unwrap().query().map(|x| x.to_owned() ),
                expected: Some(String::from("name=bob"))
            },
            TestCase{
                case: Uri::parse("http://example.com?name=bob&age=21").unwrap().query().map(|x| x.to_owned() ),
                expected: Some(String::from("name=bob&age=21"))
            },
            TestCase{
                case: Uri::parse("http://example.com?name=bob&age=21#page1").unwrap().query().map(|x| x.to_owned() ),
                expected: Some(String::from("name=bob&age=21"))
            },
            TestCase{
                case: Uri::parse("http://example.com?#page1").unwrap().query().map(|x| x.to_owned() ),
                expected: Some(String::from(""))
            },
            TestCase{
                case: Uri::parse("http://example.com?:/abc").unwrap().query().map(|x| x.to_owned() ),
                expected: Some(String::from(":/abc"))
            },
            TestCase{
                case: Uri::parse("http://example.com?(xyz)/").unwrap().query().map(|x| x.to_owned() ),
                expected: Some(String::from("(xyz)/"))
            },  
        ];
        for test in tests.iter() {
            assert_eq!(test.case, test.expected)
        }
    }


    #[test]
    fn uri_stringify_ok() {
        let tests = [
            TestCase{
                case: Uri::parse("http://example.com:8080").unwrap().stringify().unwrap(),
                expected: String::from("http://example.com:8080"),
            },
            TestCase{
                case: Uri::parse("http://example.com:8080/").unwrap().stringify().unwrap(),
                expected: String::from("http://example.com:8080/"),
            },
            TestCase{
                case: Uri::parse("http://user@example.com:8080").unwrap().stringify().unwrap(),
                expected: String::from("http://user@example.com:8080"),
            },
            TestCase{
                case: Uri::parse("http://user@example.com:8080/this/is/a/path").unwrap().stringify().unwrap(),
                expected: String::from("http://user@example.com:8080/this/is/a/path"),
            },
            TestCase{
                case: Uri::parse("http://user@example.com:8080/this/is/a/path?name=tom").unwrap().stringify().unwrap(),
                expected: String::from("http://user@example.com:8080/this/is/a/path?name=tom"),
            },
            TestCase{
                case: Uri::parse("http://user@example.com:8080/this/is/a/path?name=tom#page3").unwrap().stringify().unwrap(),
                expected: String::from("http://user@example.com:8080/this/is/a/path?name=tom#page3"),
            },
            TestCase{
                case: Uri::parse("http://user@example.com:8080/this/is%20a/path?name=tom#page3").unwrap().stringify().unwrap(),
                expected: String::from("http://user@example.com:8080/this/is%20a/path?name=tom#page3"),
            },
            TestCase{
                case: Uri::parse("http://[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080/this/is%20a/path?name=tom#page3").unwrap().stringify().unwrap(),
                expected: String::from("http://[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080/this/is%20a/path?name=tom#page3"),
            },
            TestCase{
                case: Uri::parse("//[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080/this/is%20a/path?name=tom#page3").unwrap().stringify().unwrap(),
                expected: String::from("//[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080/this/is%20a/path?name=tom#page3"),
            },
            TestCase{
                case: Uri::parse("//:8080/this/is%20a/path?name=tom#page3").unwrap().stringify().unwrap(),
                expected: String::from("//:8080/this/is%20a/path?name=tom#page3"),
            },
            TestCase{
                case: Uri::parse("//:/this/is%20a/path?name=tom#page3").unwrap().stringify().unwrap(),
                expected: String::from("/this/is%20a/path?name=tom#page3"),
            },
            TestCase{
                case: Uri::parse("//:/this/is%20a/path?name=tom#").unwrap().stringify().unwrap(),
                expected: String::from("/this/is%20a/path?name=tom#"),
            },
            TestCase{
                case: Uri::parse("/this/is%20a/path?").unwrap().stringify().unwrap(),
                expected: String::from("/this/is%20a/path?"),
            },
            TestCase{
                case: Uri::parse("/this/is%20a/path").unwrap().stringify().unwrap(),
                expected: String::from("/this/is%20a/path"),
            },
        ];


        for test in tests.iter() {
            assert_eq!(test.case, test. expected)
        }
    }

}
