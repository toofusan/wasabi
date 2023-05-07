use crate::net::udp::UdpPacket;
use crate::net::udp::UDP_PORT_DHCP_CLIENT;
use crate::net::udp::UDP_PORT_DHCP_SERVER;
use crate::net::EthernetAddr;
use crate::net::EthernetType;
use crate::net::IpV4Addr;
use crate::net::IpV4Packet;
use crate::net::IpV4Protocol;
use crate::util::Sliceable;
use core::mem::size_of;
use core::mem::MaybeUninit;

// https://datatracker.ietf.org/doc/html/rfc2132
// 3.3. Subnet Mask (len = 4)
pub const DHCP_OPT_NETMASK: u8 = 1;
// 3.5. Router Option (len = 4 * n where n >= 1)
pub const DHCP_OPT_ROUTER: u8 = 3;
// 3.8. Domain Name Server Option (len = 4 * n where n >= 1)
pub const DHCP_OPT_DNS: u8 = 6;
// 9.6. DHCP Message Type (len = 1)
pub const DHCP_OPT_MESSAGE_TYPE: u8 = 53;
pub const DHCP_OPT_MESSAGE_TYPE_DISCOVER: u8 = 1;
pub const DHCP_OPT_MESSAGE_TYPE_OFFER: u8 = 2;
pub const DHCP_OPT_MESSAGE_TYPE_ACK: u8 = 5;

// https://datatracker.ietf.org/doc/html/rfc2131#section-2
pub const DHCP_OP_BOOTREQUEST: u8 = 1; // CLIENT -> SERVER
pub const DHCP_OP_BOOTREPLY: u8 = 2; // SERVER -> CLIENT

#[repr(packed)]
#[allow(unused)]
#[derive(Copy, Clone)]
pub struct DhcpPacket {
    udp: UdpPacket,
    op: u8,
    htype: u8,
    hlen: u8,
    hops: u8,
    xid: u32,
    secs: u16,
    flags: u16,
    ciaddr: IpV4Addr,
    yiaddr: IpV4Addr,
    siaddr: IpV4Addr,
    giaddr: IpV4Addr,
    chaddr: EthernetAddr,
    chaddr_padding: [u8; 10],
    sname: [u8; 64],
    file: [u8; 128],
    cookie: [u8; 4],
    // Optional fields follow
}
const _: () = assert!(size_of::<DhcpPacket>() == 282);
impl DhcpPacket {
    pub fn op(&self) -> u8 {
        self.op
    }
    pub fn yiaddr(&self) -> IpV4Addr {
        self.yiaddr
    }
    pub fn request(src_eth_addr: EthernetAddr) -> Self {
        let mut this = Self::default();
        // eth
        this.udp.ip.eth.dst = EthernetAddr::broardcast();
        this.udp.ip.eth.src = src_eth_addr;
        this.udp.ip.eth.eth_type = EthernetType::ip_v4();
        // ip
        this.udp.ip.version_and_ihl = 0x45; // IPv4, header len = 5 * sizeof(uint32_t) = 20 bytes
        this.udp
            .ip
            .set_data_length((size_of::<Self>() - size_of::<IpV4Packet>()) as u16);
        this.udp.ip.ident = 0x426b;
        this.udp.ip.ttl = 0xff;
        this.udp.ip.protocol = IpV4Protocol::udp();
        this.udp.ip.dst = IpV4Addr::broardcast();
        this.udp.ip.calc_checksum();
        // udp
        this.udp.set_src_port(UDP_PORT_DHCP_CLIENT);
        this.udp.set_dst_port(UDP_PORT_DHCP_SERVER);
        this.udp
            .set_data_size((size_of::<Self>() - size_of::<UdpPacket>()) as u16);
        // dhcp
        this.op = DHCP_OP_BOOTREQUEST;
        this.htype = 1;
        this.hlen = 6;
        this.xid = 0x1234;
        this.chaddr = src_eth_addr;
        // https://datatracker.ietf.org/doc/html/rfc2132#section-2
        // 2. BOOTP Extension/DHCP Option Field Format
        // > The value of the magic cookie is the 4 octet
        // dotted decimal 99.130.83.99 ... in network byte order.
        this.cookie = [99, 130, 83, 99];
        // this.udp.csum can be 0 since it is optional
        this
    }
}
impl Default for DhcpPacket {
    fn default() -> Self {
        // SAFETY: This is safe since DhcpPacket is valid as a data for any contents
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}
unsafe impl Sliceable for DhcpPacket {}
