use serde::{Deserialize, Deserializer};
use std::{io, vec};
use std::{fs, path::PathBuf};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub upload_threshold: u32,
    pub upload_process_batch_size: usize
}

#[derive(Deserialize, Clone)]
pub struct Ssl {
    pub cert_file: PathBuf,
    pub key_file: PathBuf,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ServerMode {
    Http,
    Https,
}

impl ServerMode {
    pub fn protocol(&self) -> &str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

// Custom type for Host
#[derive(Debug, Clone)]
pub enum Host {
    Ip(IpAddr),
    Hostname(String),
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Host::Ip(ip) => ip.fmt(f),
            Host::Hostname(s) => write!(f, "{s}"),
        }
    }
}

impl<'de> Deserialize<'de> for Host {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if let Ok(address) = &s.parse::<Ipv6Addr>() {
            return Ok(Host::Ip(IpAddr::V6(*address)));
        }
        if let Ok(address) = &s.parse::<Ipv4Addr>() {
            return Ok(Host::Ip(IpAddr::V4(*address)));
        }
        Ok(Host::Hostname(s))
    }
}

// Custom type for Port
#[derive(Debug, Clone, Copy)]
pub struct Port(u16);

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Conversion from/to raw port numbers
impl From<u16> for Port {
    fn from(port: u16) -> Self {
        Port(port)
    }
}

impl From<Port> for u16 {
    fn from(port: Port) -> Self {
        port.0
    }
}

impl<'de> Deserialize<'de> for Port {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let number = u16::deserialize(d)?;
        Ok(Self(number))
    }
}

// Custom type that wraps Host and Port
pub struct HostPort<'a>(&'a Host, &'a Port);

// Implement ToSocketAddrs for the HostPort type
impl<'a> ToSocketAddrs for HostPort<'a> {
    type Iter = std::vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        match &self.0 {
            Host::Ip(ip) => {
                let socket = SocketAddr::new(*ip, self.1.0);
                Ok(vec![socket].into_iter())
            }
            Host::Hostname(hostname) => {
                let addresses = (hostname.as_str(), self.1.0).to_socket_addrs()?;
                Ok(addresses.collect::<Vec<_>>().into_iter())
            }
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub host: Host,
    pub port: Port,
    pub ssl: Ssl,
    pub mode: ServerMode,
}

impl Server {
    pub fn listen_address(&self) -> HostPort {
        HostPort(&self.host, &self.port)
    }
}

pub fn load_config() -> Config {
    let config_contents = fs::read_to_string("config.toml").expect("Failed to load config file");
    let config: Config = toml::from_str(&config_contents).expect("Failed to parse config file contents!");
    config
}
