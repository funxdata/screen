use get_if_addrs::get_if_addrs;
pub fn get_local_ip() -> Option<std::net::Ipv4Addr> {
    let interfaces = get_if_addrs().ok()?;
    for iface in interfaces {
        if !iface.is_loopback() {
            if let std::net::IpAddr::V4(ipv4) = iface.ip() {
                return Some(ipv4);
            }
        }
    }
    None
}