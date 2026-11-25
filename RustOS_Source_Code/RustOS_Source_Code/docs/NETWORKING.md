# RustOS Networking Architecture & VirtualBox Integration Guide

## 1. Overview

RustOS implements a modular bare-metal TCP/IP networking stack designed specifically for virtual machine environments such as **Oracle VirtualBox**. It provides seamless guest-to-host and guest-to-internet connectivity through VirtualBox's virtual network adapters.

```
+--------------------------------------------------------------------------+
|                       APPLICATION LAYER PROTOCOLS                        |
|        [DHCP Client]      [DNS Client]       [HTTP/1.1 Web Client]       |
+--------------------------------------------------------------------------+
|                        TRANSPORT LAYER PROTOCOLS                         |
|        [UDP - User Datagram Protocol]   [TCP - Transmission Control]     |
+--------------------------------------------------------------------------+
|                         INTERNET LAYER PROTOCOLS                         |
|   [IPv4 - Internet Protocol v4]   [ICMP - Internet Control Message / Ping]|
|   [ARP - Address Resolution Protocol & Dynamic Cache Table]              |
+--------------------------------------------------------------------------+
|                         LINK / HARDWARE DRIVER                           |
|   [Ethernet II Frame Framing (802.3 MAC Addressing & Checksumming)]      |
|   [Intel 82540EM (e1000) Gigabit PCI NIC Driver]                         |
+--------------------------------------------------------------------------+
|                     ORACLE VIRTUALBOX VIRTUAL NETWORK                    |
|   - Virtual Adapter: Intel PRO/1000 MT Desktop (82540EM)                 |
|   - Attached to: NAT (Default) or Bridged Adapter (Physical Wi-Fi/LAN)   |
+--------------------------------------------------------------------------+
```

---

## 2. VirtualBox Networking Configuration

### Physical Host vs Guest Connectivity
- **Host System**: Windows 11 host connects to the outside world via physical Wi-Fi (802.11ax/ac) or Ethernet (Gigabit RJ-45).
- **VirtualBox Hypervisor**: Acts as a virtual router/switch with built-in DHCP server and NAT engine.
- **RustOS Guest**: Sees an **Intel PRO/1000 MT Desktop (82540EM)** PCI Ethernet adapter. The host translates the guest's virtual Ethernet frames to the host's physical Wi-Fi adapter seamlessly.

### Network Modes in VirtualBox

#### Mode A: NAT (Network Address Translation) — Recommended Default
- **Guest IP Assigned via DHCP**: `10.0.2.15`
- **Virtual Router / Gateway**: `10.0.2.2`
- **Virtual DNS Server**: `10.0.2.3`
- **Host Loopback Alias**: `10.0.2.2` can be used to reach web servers running on the Windows host!
- **Internet Access**: VirtualBox forwards all guest TCP/UDP traffic through the Windows host's active internet connection.

#### Mode B: Bridged Networking (Wi-Fi or Ethernet)
- The guest adapter is bound directly to the host's physical network adapter (e.g. `Intel(R) Wi-Fi 6 AX201`).
- The guest receives an IP directly from the physical LAN/Wi-Fi router (e.g. `192.168.1.X`).
- Allows other computers and devices on the LAN to communicate directly with RustOS.

---

## 3. Intel 82540EM (e1000) Driver Implementation

The e1000 driver in `src/net/e1000.rs` communicates directly with the virtual hardware registers via PCI Memory Mapped I/O (MMIO):

1. **PCI Discovery**:
   - Scans PCI buses 0-255, devices 0-31, functions 0-7.
   - Matches Vendor ID `0x8086` and Device ID `0x100E`.
   - Reads Base Address Register 0 (BAR0) for the 128 KB MMIO register space.
   - Enables PCI Bus Mastering and MMIO memory access in the PCI Command Register.

2. **Ring Buffer Initialization**:
   - **Receive Descriptors**: 16 descriptors arranged in a ring buffer at 16-byte alignment. Each descriptor points to a 2048-byte physical DMA buffer.
   - **Transmit Descriptors**: 16 descriptors arranged in a ring buffer. Packets to be sent are copied into the DMA buffer and marked with `CMD_EOP | CMD_IFCS | CMD_RS`.

3. **MAC Address Extraction**:
   - Reads the factory MAC address from the e1000 EEPROM register (`EERD`) or internal receive address registers `RAL0` (offset `0x5400`) and `RAH0` (offset `0x5404`).
   - Standard VirtualBox default MAC address: `08:00:27:xx:xx:xx`.

4. **Link Up Verification**:
   - Reads Device Status Register (offset `0x0008`), checking bit 1 (`LU` - Link Up).

---

## 4. Protocol Implementations

### A. Ethernet II Framing
- 14-byte Ethernet header:
  - Destination MAC (6 bytes)
  - Source MAC (6 bytes)
  - EtherType (2 bytes): `0x0800` for IPv4, `0x0806` for ARP.
- Minimum payload padding to 46 bytes (60 bytes minimum frame length without FCS).

### B. ARP (Address Resolution Protocol)
- Maintains an in-kernel `ArpTable` mapping 32-bit IPv4 addresses to 48-bit MAC addresses.
- Sends broadcast ARP requests (`FF:FF:FF:FF:FF:FF`) when resolving the default gateway's MAC address.
- Handles ARP replies and caches entries dynamically.

### C. IPv4
- RFC 791 compliant header generation.
- Correct 16-bit one's complement Internet Checksum verification.
- TTL defaulted to 64.
- Dispatches payload based on protocol field:
  - `0x01`: ICMP
  - `0x06`: TCP
  - `0x11`: UDP

### D. ICMP (Ping)
- Implements ICMP Echo Request (Type 8, Code 0) and Echo Reply (Type 0, Code 0).
- Sequence number tracking and millisecond round-trip time (RTT) calculation via PIT timer.

### E. DHCP (Dynamic Host Configuration Protocol)
- UDP Client port 68, Server port 67.
- Sends `DHCP Discover` broadcast with Option 53 (`DHCPDISCOVER`), Option 55 (Parameter Request List), and Option 60 (Vendor Class Identifier).
- Parses `DHCP Offer` and commits `DHCP Request` to acquire:
  - Assigned Guest IPv4
  - Subnet Mask
  - Router / Default Gateway IP
  - Domain Name Server (DNS) IP
  - Lease Duration

### F. DNS (Domain Name System)
- UDP Client querying port 53.
- Encodes domain labels (e.g. `example.com` -> `\x07example\x03com\x00`).
- Parses Type 1 (`A` record) resource records and extracts 32-bit IPv4 addresses.

### G. TCP & HTTP
- Full 3-way handshake:
  1. RustOS transmits `SYN` segment with Initial Sequence Number (ISN).
  2. Server responds with `SYN-ACK`.
  3. RustOS replies with `ACK`.
- Transmits HTTP GET request:
  ```http
  GET / HTTP/1.1\r\n
  Host: 10.0.2.2:8080\r\n
  User-Agent: RustOS/1.0 (x86; Educational)\r\n
  Connection: close\r\n
  \r\n
  ```
- Receives inbound HTTP response stream, accumulates segments, parses headers and body.
