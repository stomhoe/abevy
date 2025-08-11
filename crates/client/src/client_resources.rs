use bevy::prelude::*;
use std::net::Ipv4Addr;
#[derive(Resource, Debug)]
pub struct TargetJoinServer {
    ip: Ipv4Addr,
    port: Option<u16>,
}

impl TargetJoinServer {
    pub fn new(ip_port: String) -> Result<Self, BevyError> {
        let parts: Vec<&str> = ip_port.split(':').collect();
        let ip = parts[0].parse::<Ipv4Addr>().map_err(|_| {
            BevyError::from(format!("Invalid IP address: {}", parts[0]))
        })?;
        let port = if parts.len() == 2 {
            Some(parts[1].parse::<u16>().map_err(|_| {
                BevyError::from(format!("Invalid port number: {}", parts[1]))
            })?)
        } else {
            None
        };
        Ok(Self { ip, port })
    }
    pub fn ip(&self) -> Ipv4Addr { self.ip }
    pub fn port(&self) -> Option<u16> { self.port }
}


impl Default for TargetJoinServer {
    fn default() -> Self {
        Self {
            ip: Ipv4Addr::UNSPECIFIED,
            port: None,
        }
    }
}

