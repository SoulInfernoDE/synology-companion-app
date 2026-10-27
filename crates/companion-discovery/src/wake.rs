//! Wake-on-LAN: the "magic packet" that switches on a NAS that is off, if DSM has
//! Wake-on-LAN enabled (Control Panel › Hardware & Power).

use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

/// "00:11:32:aa:bb:cc" or "00-11-32-AA-BB-CC" → six bytes.
fn parse_mac(mac: &str) -> Option<[u8; 6]> {
    let parts: Vec<u8> = mac
        .split([':', '-'])
        .map(|part| u8::from_str_radix(part, 16))
        .collect::<Result<_, _>>()
        .ok()?;
    parts.try_into().ok()
}

/// Six bytes of 0xFF, then the MAC address sixteen times.
fn magic_packet(mac: [u8; 6]) -> Vec<u8> {
    let mut packet = vec![0xff; 6];
    for _ in 0..16 {
        packet.extend_from_slice(&mac);
    }
    packet
}

/// Sends the magic packet for `mac` to the whole network and, when the NAS's last
/// address is known, to the broadcast address of its /24 network, on the usual
/// ports 7 and 9.
pub fn wake(mac: &str, last_address: Option<Ipv4Addr>) -> Result<(), String> {
    let mac = parse_mac(mac).ok_or_else(|| format!("invalid MAC address {mac:?}"))?;
    let packet = magic_packet(mac);
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(|error| error.to_string())?;
    socket
        .set_broadcast(true)
        .map_err(|error| error.to_string())?;

    let mut targets = vec![Ipv4Addr::BROADCAST];
    if let Some(address) = last_address {
        let [a, b, c, _] = address.octets();
        targets.push(Ipv4Addr::new(a, b, c, 255));
    }
    let mut sent = false;
    for target in targets {
        for port in [9, 7] {
            sent |= socket
                .send_to(&packet, SocketAddrV4::new(target, port))
                .is_ok();
        }
    }
    if sent {
        Ok(())
    } else {
        Err("the magic packet could not be sent".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_mac_notations() {
        let expected = Some([0x00, 0x11, 0x32, 0xaa, 0xbb, 0xcc]);
        assert_eq!(parse_mac("00:11:32:aa:bb:cc"), expected);
        assert_eq!(parse_mac("00-11-32-AA-BB-CC"), expected);
        assert_eq!(parse_mac("00:11:32"), None);
        assert_eq!(parse_mac("zz:11:32:aa:bb:cc"), None);
    }

    #[test]
    fn magic_packet_repeats_the_mac() {
        let packet = magic_packet([1, 2, 3, 4, 5, 6]);
        assert_eq!(packet.len(), 6 + 16 * 6);
        assert!(packet[..6].iter().all(|byte| *byte == 0xff));
        assert_eq!(&packet[6..12], &[1, 2, 3, 4, 5, 6]);
        assert_eq!(&packet[96..], &[1, 2, 3, 4, 5, 6]);
    }
}
