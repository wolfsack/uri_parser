use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyScheme,
    EmptyAuthority,
    ParsePortError,
    SchemeIllegalFirstCharacter,
    SchemeIllegalCharacter,
    UserinfoIllegalCharacter,
    IllegaHostDefinition,
    IllegalIPvFuture,
    IllegalIPv6,
    HostIllegalCharacter,
    PathIllegalStart,
    PathIllegalCharacter,
    QueryIllegalCharacter,
    FragmentIllegalCharacter,
    IllegalCharacter,
    IllegalPercentEncoding,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EmptyScheme => write!(f, "Empty Scheme not allowed."),
            Error::SchemeIllegalFirstCharacter => write!(f, "Illegal first charater in Scheme."),
            Error::SchemeIllegalCharacter => write!(f, "Illegal charater in Scheme."),
            Self::EmptyAuthority => {
                write!(f, "Authority is empty while indicating its existing.")
            }
            Self::ParsePortError => write!(f, "Port is not a integer 'u16'."),
            Self::IllegalCharacter => write!(f, "Found a invalid character."),
            Self::IllegalPercentEncoding => write!(f, "Illegal character after '%'."),
            Self::UserinfoIllegalCharacter => write!(f, "Illegal character in userinfo."),
            Self::IllegaHostDefinition => write!(f, "Host syntax is invalid."),
            Self::IllegalIPvFuture => write!(f, "IPvFuture syntax is invalid."),
            Self::IllegalIPv6 => write!(f, "IPv6 syntax is invalid."),
            Self::HostIllegalCharacter => write!(f, "Illegal character in Host."),
            Self::PathIllegalStart => write!(f, "Path can't start with '//'."),
            Self::PathIllegalCharacter => write!(f, "Illegal character in Path."),
            Self::QueryIllegalCharacter => write!(f, "Illegal character in Query."),
            Self::FragmentIllegalCharacter => write!(f, "Illegal character in Fragment."),
        }
    }
}
