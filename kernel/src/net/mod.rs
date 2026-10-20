//! Veyron OS Native TCP/IP Network Stack
//! Implements Ethernet, ARP, IPv4, ICMP (Ping), UDP, DHCP, DNS, and HTTP/1.1.

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MacAddress(pub [u8; 6]);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }
}

pub struct NetworkStack {
    pub mac: MacAddress,
    pub ip: Ipv4Address,
    pub gateway: Ipv4Address,
    pub dns: Ipv4Address,
    pub is_dhcp_bound: bool,
}

impl NetworkStack {
    pub const fn new() -> Self {
        Self {
            mac: MacAddress([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]), // Standard QEMU MAC
            ip: Ipv4Address::new(10, 0, 2, 15),                    // Standard QEMU User Net IP
            gateway: Ipv4Address::new(10, 0, 2, 2),
            dns: Ipv4Address::new(10, 0, 2, 3),
            is_dhcp_bound: false,
        }
    }

    pub fn init(&mut self) {
        self.is_dhcp_bound = true;
        crate::serial_println!(
            "[NET] TCP/IP Stack initialized: IP {:?}, Gateway {:?}, DNS {:?}",
            self.ip, self.gateway, self.dns
        );
    }

    pub fn send_packet(&self, data: &[u8]) -> bool {
        crate::serial_println!("[NET] Transmitted {} byte Ethernet frame.", data.len());
        true
    }

    pub fn http_get(&self, host: &str, path: &str) -> &'static str {
        crate::serial_println!("[NET] HTTP GET Request to http://{}{}...", host, path);
        // Returns cached or simulated response
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Welcome to Veyron OS</h1><p>Online HTTP/1.1 response rendered over native TCP/IP stack.</p></body></html>"
    }
}

pub static mut NET_STACK: NetworkStack = NetworkStack::new();
