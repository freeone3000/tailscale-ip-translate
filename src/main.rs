mod args;

use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

fn main() {
    let opt = match args::parse() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    if opt.reverse {
        println!("{}", six_to_four(&opt.addr).expect("Failed to convert IPv6 to IPv4"));
    } else {
        println!("{}", four_to_six(&opt.addr, opt.translator_id).expect("Failed to convert IPv4 to IPv6"));
    }
}

fn six_to_four(addr_str: &String) -> Result<Ipv4Addr, std::net::AddrParseError> {
    // it's not an RFC-complaint IPv4-as-IPv6 address, it's a 4via6 address, so we truncate.
    let addr = Ipv6Addr::from_str(addr_str)?.to_bits();
    Ok(Ipv4Addr::from_bits(addr as u32))
}

fn four_to_six(addr_str: &String, translator_id: u32) -> Result<Ipv6Addr, std::net::AddrParseError> {
    let addr = Ipv4Addr::from_str(addr_str)?;

    // according to rust spec, narrowing conversions truncate. no need to mask.
    let mut prefix: [u8; 16] = [0xfd, 0x7a, 0x11, 0x5c, 0xa1, 0xe0, 0xb, 0x1a,
        0, 0, 0, 0, 0, 0, 0, 0];
    prefix[8..12].copy_from_slice(&translator_id.to_be_bytes());
    prefix[12..16].copy_from_slice(&addr.octets());

    Ok(Ipv6Addr::from(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_to_six_basic() {
        let result = four_to_six(&"192.168.1.1".to_string(), 7).expect("Failed to convert");
        let expected_bytes: [u8; 16] = [
            0xfd, 0x7a, 0x11, 0x5c, 0xa1, 0xe0, 0x0b, 0x1a,
            0x00, 0x00, 0x00, 0x07, 0xc0, 0xa8, 0x01, 0x01,
        ];
        assert_eq!(result, Ipv6Addr::from(expected_bytes));
    }

    #[test]
    fn test_four_to_six_different_translator_id() {
        let result = four_to_six(&"10.0.0.1".to_string(), 42).expect("Failed to convert");
        let expected_bytes: [u8; 16] = [
            0xfd, 0x7a, 0x11, 0x5c, 0xa1, 0xe0, 0x0b, 0x1a,
            0x00, 0x00, 0x00, 0x2a, 0x0a, 0x00, 0x00, 0x01,
        ];
        assert_eq!(result, Ipv6Addr::from(expected_bytes));
    }

    #[test]
    fn test_six_to_four_basic() {
        let ipv6_str = "fd7a:115c:a1e0:b1a::7:c0a8:101".to_string();
        let result = six_to_four(&ipv6_str).expect("Failed to convert");
        assert_eq!(result, Ipv4Addr::new(192, 168, 1, 1));
    }

    #[test]
    fn test_round_trip_ipv4_to_ipv6_to_ipv4() {
        let original_ip = "172.16.0.1".to_string();
        let translator_id = 7;

        // Convert IPv4 -> IPv6
        let ipv6 = four_to_six(&original_ip, translator_id).expect("Failed to convert IPv4 to IPv6");
        let ipv6_str = ipv6.to_string();

        // Convert IPv6 -> IPv4
        let recovered_ip = six_to_four(&ipv6_str).expect("Failed to convert IPv6 to IPv4");

        // Parse original to compare
        let original_parsed = Ipv4Addr::from_str(&original_ip).expect("Failed to parse original IP");
        assert_eq!(recovered_ip, original_parsed);
    }

    #[test]
    fn test_round_trip_range() {
        let translator_id = 7;

        // Test a range of IPv4 addresses
        for a in 0..=255 {
            for b in 0..=255 {
                let ip_str = format!("10.0.{}.{}", a, b);

                // Convert IPv4 -> IPv6
                let ipv6 = four_to_six(&ip_str, translator_id)
                    .expect(&format!("Failed to convert IPv4 to IPv6: {}", ip_str));
                let ipv6_str = ipv6.to_string();

                // Convert IPv6 -> IPv4
                let recovered_ip = six_to_four(&ipv6_str)
                    .expect(&format!("Failed to convert IPv6 to IPv4: {}", ipv6_str));

                // Parse original to compare
                let original_parsed = Ipv4Addr::from_str(&ip_str)
                    .expect(&format!("Failed to parse original IP: {}", ip_str));
                assert_eq!(recovered_ip, original_parsed, "Round-trip failed for {}", ip_str);
            }
        }
    }

    #[test]
    fn test_round_trip_different_translator_ids() {
        let ipv4_str = "8.8.8.8".to_string();

        // Test multiple translator IDs
        for translator_id in [0, 1, 7, 42, 255, 1000, u32::MAX].iter() {
            let ipv6 = four_to_six(&ipv4_str, *translator_id)
                .expect(&format!("Failed to convert with translator_id: {}", translator_id));
            let ipv6_str = ipv6.to_string();

            let recovered_ip = six_to_four(&ipv6_str)
                .expect(&format!("Failed to recover from translator_id: {}", translator_id));

            let original_parsed = Ipv4Addr::from_str(&ipv4_str).expect("Failed to parse IP");
            assert_eq!(recovered_ip, original_parsed, "Round-trip failed for translator_id: {}", translator_id);
        }
    }

    #[test]
    fn test_correctness_specific_ips() {
        let test_cases = vec![
            ("0.0.0.0", 7),
            ("127.0.0.1", 7),
            ("255.255.255.255", 7),
            ("192.168.1.1", 7),
            ("10.20.30.40", 42),
            ("172.31.255.255", 100),
        ];

        for (ip_str, translator_id) in test_cases {
            let ipv4_original = Ipv4Addr::from_str(ip_str).expect("Failed to parse test IP");

            // Convert to IPv6
            let ipv6 = four_to_six(&ip_str.to_string(), translator_id)
                .expect(&format!("Failed to convert {}", ip_str));

            // Check that the IPv6 address has the correct prefix
            let bytes = ipv6.segments();
            assert_eq!(bytes[0], 0xfd7a, "Incorrect prefix for {}", ip_str);

            // Check that the translator ID is in the correct position
            let ipv6_bytes = ipv6.octets();
            let translator_id_from_ipv6 = u32::from_be_bytes([
                ipv6_bytes[8],
                ipv6_bytes[9],
                ipv6_bytes[10],
                ipv6_bytes[11],
            ]);
            assert_eq!(translator_id_from_ipv6, translator_id, "Incorrect translator ID for {}", ip_str);

            // Check that the IPv4 address is in the correct position
            let ipv4_from_ipv6 = Ipv4Addr::new(
                ipv6_bytes[12],
                ipv6_bytes[13],
                ipv6_bytes[14],
                ipv6_bytes[15],
            );
            assert_eq!(ipv4_from_ipv6, ipv4_original, "Incorrect IPv4 in IPv6 for {}", ip_str);

            // Round-trip test
            let ipv6_str = ipv6.to_string();
            let recovered_ipv4 = six_to_four(&ipv6_str)
                .expect(&format!("Failed to recover from IPv6 for {}", ip_str));
            assert_eq!(recovered_ipv4, ipv4_original, "Round-trip failed for {}", ip_str);
        }
    }
}
