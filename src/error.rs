// SPDX-License-Identifier: Apache-2.0 AND MIT
//! `Error` enum type

/// An error enum.
pub enum Error {
    /// [lapin::Error] variant.
    ///
    /// [lapin::Error]: https://docs.rs/lapin/latest/lapin/enum.Error.html
    Internal(lapin::Error),
    /// Other error variant.
    Other,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Internal(err) => Some(err),
            Self::Other => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal(err) => err.fmt(f),
            Self::Other => write!(f, "other error"),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal(err) => err.fmt(f),
            Self::Other => write!(f, "Error::Other"),
        }
    }
}

impl From<lapin::Error> for Error {
    fn from(err: lapin::Error) -> Self {
        Self::Internal(err)
    }
}

impl std::cmp::PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Internal(err) => match other {
                Self::Internal(other) => Self::eq_internal(err, other),
                _ => false,
            },
            Self::Other => match other {
                Self::Other => true,
                _ => false,
            },
        }
    }
}

type LapinErr = lapin::Error;

impl Error {
    fn eq_internal(a: &lapin::Error, b: &lapin::Error) -> bool {
        match a {
            LapinErr::InvalidMethod(a) => match b {
                LapinErr::InvalidMethod(b) => a == b,
                _ => false,
            },
            LapinErr::InvalidChannel(a) => match b {
                LapinErr::InvalidChannel(b) => a == b,
                _ => false,
            },
            LapinErr::InvalidAck => match b {
                LapinErr::InvalidAck => true,
                _ => false,
            },
            LapinErr::InvalidBodyReceived => match b {
                LapinErr::InvalidBodyReceived => true,
                _ => false,
            },
            LapinErr::InvalidFrameReceived => match b {
                LapinErr::InvalidFrameReceived => true,
                _ => false,
            },
            LapinErr::UnexpectedReply => match b {
                LapinErr::UnexpectedReply => true,
                _ => false,
            },
            LapinErr::ChannelsLimitReached => match b {
                LapinErr::ChannelsLimitReached => true,
                _ => false,
            },
            LapinErr::InvalidChannelState(a) => match b {
                LapinErr::InvalidChannelState(b) => a == b,
                _ => false,
            },
            LapinErr::InvalidConnectionState(a) => match b {
                LapinErr::InvalidConnectionState(b) => a == b,
                _ => false,
            },
            LapinErr::ParsingError(a) => match b {
                LapinErr::ParsingError(b) => a == b,
                _ => false,
            },
            LapinErr::SerialisationError(a) => match b {
                LapinErr::SerialisationError(b) => Self::eq_gen_error(a, b),
                _ => false,
            },
            LapinErr::IOError(a) => match b {
                LapinErr::IOError(b) => a.kind() == b.kind(),
                _ => false,
            },
            LapinErr::ProtocolError(a1, a2) => match b {
                LapinErr::ProtocolError(b1, b2) => a1 == b1 && a2 == b2,
                _ => false,
            },
            LapinErr::__Nonexhaustive => match b {
                LapinErr::__Nonexhaustive => true,
                _ => false,
            },
        }
    }
    fn eq_gen_error(a: &cookie_factory::GenError, b: &cookie_factory::GenError) -> bool {
        match a {
            cookie_factory::GenError::BufferTooSmall(a) => match b {
                cookie_factory::GenError::BufferTooSmall(b) => a == b,
                _ => false,
            },
            cookie_factory::GenError::BufferTooBig(a) => match b {
                cookie_factory::GenError::BufferTooBig(b) => a == b,
                _ => false,
            },
            cookie_factory::GenError::InvalidOffset => match b {
                cookie_factory::GenError::InvalidOffset => true,
                _ => false,
            },
            cookie_factory::GenError::IoError(a) => match b {
                cookie_factory::GenError::IoError(b) => a.kind() == b.kind(),
                _ => false,
            },
            cookie_factory::GenError::CustomError(a) => match b {
                cookie_factory::GenError::CustomError(b) => a == b,
                _ => false,
            },
            cookie_factory::GenError::NotYetImplemented => match b {
                cookie_factory::GenError::NotYetImplemented => true,
                _ => false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use cookie_factory;
    use lapin;
    use std::error::Error;
    use std::fmt::Write;
    use std::io::{self, ErrorKind};
    #[test]
    fn source_internal_then_debug() {
        struct Test {
            data: super::Error,
            want: String,
        }
        let mut tests = [
            Test {
                data: crate::Error::Internal(lapin::Error::UnexpectedReply),
                want: String::from("UnexpectedReply"),
            },
            Test {
                data: crate::Error::Internal(lapin::Error::ChannelsLimitReached),
                want: String::from("ChannelsLimitReached"),
            },
        ];
        for t in &mut tests {
            let mut got = String::new();
            let err = t.data.source().unwrap();
            write!(&mut got, "{:?}", err).unwrap();
            assert_eq!(t.want, got);
        }
    }
    #[test]
    fn debug_internal() {
        struct Test {
            data: super::Error,
            want: String,
        }
        let mut tests = [
            Test {
                data: crate::Error::Internal(lapin::Error::UnexpectedReply),
                want: String::from("UnexpectedReply"),
            },
            Test {
                data: crate::Error::Internal(lapin::Error::ChannelsLimitReached),
                want: String::from("ChannelsLimitReached"),
            },
        ];
        for t in &mut tests {
            let mut got = String::new();
            write!(&mut got, "{:?}", t.data).unwrap();
            assert_eq!(t.want, got);
        }
    }
    #[test]
    fn from_internal() {
        use std::sync::Arc;
        struct Test {
            data: Option<lapin::Error>,
            want: crate::Error,
        }
        let mut tests = [
            Test {
                data: Some(lapin::Error::UnexpectedReply),
                want: crate::Error::Internal(lapin::Error::UnexpectedReply),
            },
            Test {
                data: Some(lapin::Error::ChannelsLimitReached),
                want: crate::Error::Internal(lapin::Error::ChannelsLimitReached),
            },
            Test {
                data: Some(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::Initial,
                )),
                want: crate::Error::Internal(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::Initial,
                )),
            },
            Test {
                data: Some(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::Connected,
                )),
                want: crate::Error::Internal(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::Connected,
                )),
            },
            Test {
                data: Some(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::Closing,
                )),
                want: crate::Error::Internal(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::Closing,
                )),
            },
            Test {
                data: Some(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::Closed,
                )),
                want: crate::Error::Internal(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::Closed,
                )),
            },
            Test {
                data: Some(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::Error,
                )),
                want: crate::Error::Internal(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::Error,
                )),
            },
            Test {
                data: Some(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::SendingContent(1024),
                )),
                want: crate::Error::Internal(lapin::Error::InvalidChannelState(
                    lapin::ChannelState::SendingContent(1024),
                )),
            },
            Test {
                data: Some(lapin::Error::InvalidConnectionState(
                    lapin::ConnectionState::Initial,
                )),
                want: crate::Error::Internal(lapin::Error::InvalidConnectionState(
                    lapin::ConnectionState::Initial,
                )),
            },
            Test {
                data: Some(lapin::Error::InvalidConnectionState(
                    lapin::ConnectionState::Connected,
                )),
                want: crate::Error::Internal(lapin::Error::InvalidConnectionState(
                    lapin::ConnectionState::Connected,
                )),
            },
            Test {
                data: Some(lapin::Error::InvalidConnectionState(
                    lapin::ConnectionState::Closing,
                )),
                want: crate::Error::Internal(lapin::Error::InvalidConnectionState(
                    lapin::ConnectionState::Closing,
                )),
            },
            Test {
                data: Some(lapin::Error::InvalidConnectionState(
                    lapin::ConnectionState::Closed,
                )),
                want: crate::Error::Internal(lapin::Error::InvalidConnectionState(
                    lapin::ConnectionState::Closed,
                )),
            },
            Test {
                data: Some(lapin::Error::InvalidConnectionState(
                    lapin::ConnectionState::Error,
                )),
                want: crate::Error::Internal(lapin::Error::InvalidConnectionState(
                    lapin::ConnectionState::Error,
                )),
            },
            Test {
                data: Some(lapin::Error::ParsingError(String::from("parse error"))),
                want: crate::Error::Internal(lapin::Error::ParsingError(String::from(
                    "parse error",
                ))),
            },
            Test {
                data: Some(lapin::Error::SerialisationError(Arc::new(
                    cookie_factory::GenError::BufferTooSmall(1),
                ))),
                want: crate::Error::Internal(lapin::Error::SerialisationError(Arc::new(
                    cookie_factory::GenError::BufferTooSmall(1),
                ))),
            },
            Test {
                data: Some(lapin::Error::SerialisationError(Arc::new(
                    cookie_factory::GenError::BufferTooBig(1024 * 1024 * 1024),
                ))),
                want: crate::Error::Internal(lapin::Error::SerialisationError(Arc::new(
                    cookie_factory::GenError::BufferTooBig(1024 * 1024 * 1024),
                ))),
            },
            Test {
                data: Some(lapin::Error::SerialisationError(Arc::new(
                    cookie_factory::GenError::InvalidOffset,
                ))),
                want: crate::Error::Internal(lapin::Error::SerialisationError(Arc::new(
                    cookie_factory::GenError::InvalidOffset,
                ))),
            },
            Test {
                data: Some(lapin::Error::SerialisationError(Arc::new(
                    cookie_factory::GenError::IoError(cookie_factory::lib::std::io::Error::new(
                        io::ErrorKind::NotFound,
                        "not found",
                    )),
                ))),
                want: crate::Error::Internal(lapin::Error::SerialisationError(Arc::new(
                    cookie_factory::GenError::IoError(cookie_factory::lib::std::io::Error::new(
                        io::ErrorKind::NotFound,
                        "not found",
                    )),
                ))),
            },
            Test {
                data: Some(lapin::Error::IOError(Arc::new(io::Error::new(
                    ErrorKind::Interrupted,
                    "interrupted",
                )))),
                want: crate::Error::Internal(lapin::Error::IOError(Arc::new(io::Error::new(
                    ErrorKind::Interrupted,
                    "interrupted",
                )))),
            },
        ];
        for t in &mut tests {
            let err = t.data.take().unwrap();
            let got = crate::Error::from(err);
            assert_eq!(t.want, got);
        }
    }
}
