use net2::UdpBuilder;
#[cfg(not(windows))]
use net2::unix::UnixUdpBuilderExt;
use std::io;
use tokio;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use super::MDNS_PORT;

#[cfg(not(windows))]
#[allow(unused_attributes)]
#[path = "net.rs"]
use net;

#[cfg(windows)]
#[allow(unused_attributes)]
#[path = "netwin.rs"]
use net;

pub enum Inet {}
pub enum Inet6 {}

pub trait AddressFamily {
    fn bind() -> io::Result<UdpSocket> {
        let addr = SocketAddr::new(Self::any_addr(), MDNS_PORT);
        let builder = Self::socket_builder()?;
        builder.reuse_address(true)?;
        #[cfg(not(windows))]
        {
            if let Err(e) = builder.reuse_port(true) {
                eprintln!("Failed to reuse_port")
            }
        }
        let socket = builder.bind(&addr)?;
        Ok(socket)
    }

    fn socket_builder() -> io::Result<UdpBuilder>;
    fn any_addr() -> IpAddr;
    fn mdns_group() -> IpAddr;
    fn join_multicast(socket: &tokio::net::UdpSocket);
    fn v6() -> bool;
}

impl AddressFamily for Inet {
    fn socket_builder() -> io::Result<UdpBuilder> {
        UdpBuilder::new_v4()
    }

    fn any_addr() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(0,0,0,0))
    }

    fn mdns_group() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(224,0,0,251))
    }

    fn join_multicast(socket: &tokio::net::UdpSocket) {
        let mut joined = false;
        for iface in net::getifaddrs() {
            match iface.ip() {
                Some(IpAddr::V4(ip)) => {
                    if ip.is_loopback() {
                        continue;
                    }
                    let _ = socket.join_multicast_v4(
                        &Ipv4Addr::new(224,0,0,251),
                        &ip)
                        .map_err(|error| {
                            if error.kind() != io::ErrorKind::AddrInUse {
                                trace!(
                                    "Failed to join to the IPv4 multicast group on interface {} with index {}: {}",
                                    ip,
                                    iface.index(),
                                    error.to_string(),
                                );
                            };
                        })
                        .map(|_| {
                            trace!(
                                "Joined to the IPv4 multicast group on interface {} with index {}",
                                ip,
                                iface.index(),
                            );
                            joined = true;
                        });
                },
                _ => continue,
            };
        };
        if !joined {
            let _ = socket.join_multicast_v4(
                &Ipv4Addr::new(224,0,0,251),
                &Ipv4Addr::new(0,0,0,0)
            ).map_err(|error| {
                if error.kind() != io::ErrorKind::AddrInUse {
                    trace!("Failed to join on 0.0.0.0 as well: {}", error.to_string());
                };
            });
        };
    }

    fn v6() -> bool {
        false
    }
}

impl AddressFamily for Inet6 {
    fn socket_builder() -> io::Result<UdpBuilder> {
        UdpBuilder::new_v6()
    }

    fn any_addr() -> IpAddr {
        IpAddr::V6(Ipv6Addr::new(0,0,0,0,0,0,0,0))
    }

    fn mdns_group() -> IpAddr {
        IpAddr::V6(Ipv6Addr::new(0xff02,0,0,0,0,0,0,0xfb))
    }

    fn join_multicast(socket: &tokio::net::UdpSocket) {
        let mut joined = false;
        for iface in net::getifaddrs() {
            match iface.ip() {
                Some(IpAddr::V6(ip)) => {
                    if ip.is_loopback() {
                        continue;
                    }
                    let _ = socket.join_multicast_v6(
                        &Ipv6Addr::new(0xff02,0,0,0,0,0,0,0xfb),
                        iface.index())
                        .map_err(|error| {
                            if error.kind() != io::ErrorKind::AddrInUse {
                                trace!(
                                    "Failed to join to the IPv6 multicast group on interface {} with index {}: {}",
                                    ip,
                                    iface.index(),
                                    error.to_string(),
                                );
                            };
                        })
                        .map(|_| {
                            trace!(
                                "Joined to the IPv6 multicast group on interface {} with index {}",
                                ip,
                                iface.index(),
                            );
                            joined = true;
                        });
                },
                _ => continue,
            };
        };
        if !joined {
            let _ = socket.join_multicast_v6(
                &Ipv6Addr::new(0xff02,0,0,0,0,0,0,0xfb),
                0u32,
            ).map_err(|error| {
                if error.kind() != io::ErrorKind::AddrInUse {
                    trace!("Failed to join on :: as well: {}", error.to_string());
                };
            });
        };
    }

    fn v6() -> bool {
        true
    }
}
