pub enum VersionType {
    Socks4 = 4,
    Socks5 = 5,
    SocksReply = 0,
}

#[derive(PartialEq)]
pub enum RequestType {
    Unknown = 0,
    Connect = 1,
    Bind = 2,
}

impl From<u8> for RequestType {
    fn from(v: u8) -> Self {
        match v {
            1 => RequestType::Connect,
            2 => RequestType::Bind,
            _ => RequestType::Unknown,
        }
    }
}

pub enum ReplyType {
    Granted = 90,
    Rejected = 91,
    NoIndented = 92,
    InvalidUser = 93,
}
