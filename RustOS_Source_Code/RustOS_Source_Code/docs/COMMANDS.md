# RustOS Shell Command Reference Manual

RustOS includes a complete 33-command Unix-compatible interactive command shell. It features command line editing, history, argument validation, and informative error reporting.

---

## Command Summary Table

| Command | Category | Syntax | Description |
|---|---|---|---|
| `help` | System | `help [cmd]` | Displays summary of all commands or detailed help on a specific command. |
| `clear` | System | `clear` | Clears the 80x25 VGA terminal screen and repositions cursor at (0, 0). |
| `reboot` | System | `reboot` | Performs hardware warm reset via PS/2 8042 keyboard controller (pulse reset line). |
| `shutdown` | System | `shutdown` | Shuts down VirtualBox guest via ACPI or QEMU/Bochs/VBox I/O port `0x604`. |
| `date` | System | `date` | Displays current system date and UTC time from CMOS Real-Time Clock (RTC). |
| `cal` | System | `cal [month] [year]` | Displays ASCII calendar grid for the current or specified month and year. |
| `echo` | Utility | `echo [text...]` | Prints text arguments to stdout with escape sequences and variable expansion. |
| `ls` | File System | `ls [-l]` | Lists all files in the current RAMFS directory with size and permissions. |
| `cat` | File System | `cat <filename>` | Concatenates and prints the contents of a specified file. |
| `touch` | File System | `touch <filename>` | Creates a new empty file or updates the timestamp of an existing file. |
| `rm` | File System | `rm <filename>` | Deletes a file from the RAMFS filesystem. |
| `cp` | File System | `cp <source> <dest>` | Copies the contents of `<source>` file to `<dest>`. |
| `mv` | File System | `mv <old> <new>` | Renames `<old>` file to `<new>`. |
| `chmod` | File System | `chmod <octal> <file>` | Updates file permissions (e.g. `chmod 755 script.sh`). |
| `wc` | Text Processing | `wc <filename>` | Counts lines, words, and byte count of a file. |
| `head` | Text Processing | `head [-n lines] <file>` | Displays the first N lines (default 10) of a text file. |
| `tail` | Text Processing | `tail [-n lines] <file>` | Displays the last N lines (default 10) of a text file. |
| `cut` | Text Processing | `cut -c <N-M> <file>` | Extracts specified character columns from each line of a file. |
| `grab` | Text Processing | `grab <pattern> <file>` | Searches for text occurrences (grep equivalent) within a file. |
| `vi` | Editor | `vi <filename>` | Full-screen interactive modal text editor (Command and Insert modes). |
| `bc` | Utility | `bc <expression>` | Command-line integer calculator evaluating arithmetic expressions. |
| `whoami` | System Info | `whoami` | Displays current active user account (`root`). |
| `uname` | System Info | `uname [-a]` | Displays operating system name, release, architecture, and version. |
| `hostname` | System Info | `hostname [new_name]` | Displays or sets the guest network hostname (default `rustos-vm`). |
| `uptime` | System Info | `uptime` | Displays system uptime in days, hours, minutes, and seconds from PIT ticks. |
| `lp` | Printing | `lp <filename>` | Formats and dumps file contents as a formatted line-printer output to screen. |
| `browser` | Web / Network | `browser [url]` | Launches the built-in text-mode web browser with optional starting URL. |
| `net` | Networking | `net` | Displays network adapter status, MAC address, link speed, and packet counters. |
| `ipconfig` | Networking | `ipconfig` | Displays IPv4 address, subnet mask, gateway, DNS server, and DHCP lease status. |
| `ping` | Networking | `ping <ip_or_host>` | Sends ICMP Echo Requests and calculates round-trip time (RTT). |
| `httpget` | Networking | `httpget <url>` | Performs an HTTP 1.1 GET request and displays response headers and body. |
| `research` | Research Lab | `research` | Opens the Deadlock Detection Research Lab benchmark suite. |
| `deadlock` | Research Lab | `deadlock` | Alias for `research` - launches the Deadlock Detection Research Lab. |

---

## Detailed Command Documentation

### 1. `help [command]`
- **Syntax**: `help` or `help <command>`
- **Description**: Displays the complete list of available shell commands with one-line descriptions. When an argument is supplied, detailed syntax and parameter documentation is presented.
- **Example**:
  ```bash
  rustos# help ping
  Usage: ping <ip_address | hostname>
  Sends 4 ICMP Echo Requests to the target IPv4 address and reports RTT.
  ```

### 2. `ls [-l]`
- **Syntax**: `ls` or `ls -l`
- **Description**: Inspects RAMFS and prints all active files. The `-l` flag provides long-format listing including octal permissions (e.g. `0644`), byte size, and filename.
- **Example**:
  ```bash
  rustos# ls -l
  -rw-r--r--   78 B   readme.txt
  -rw-r--r--   82 B   notes.txt
  -rw-r--r--   75 B   todo.txt
  -r--r--r--   68 B   about.txt
  Total: 4 files (303 bytes used)
  ```

### 3. `vi <filename>`
- **Syntax**: `vi <filename>`
- **Description**: Full-screen text editor running directly inside the 80x25 VGA buffer.
- **Modes**:
  - **Command Mode**: Navigation via Arrow Keys, `i` to enter Insert Mode, `x` to delete character, `dd` to delete line, `:w` to write/save file to RAMFS, `:q` to quit, `:wq` to save and quit.
  - **Insert Mode**: Type text directly into buffer; press `Esc` to return to Command Mode.

### 4. `ping <ip_address | domain>`
- **Syntax**: `ping 10.0.2.2` or `ping example.com`
- **Description**: Resolves domain name via DNS if necessary, constructs an ICMP Echo Request frame, transmits it across the Intel 82540EM NIC to the VirtualBox NAT gateway, and measures response time using the 100 Hz PIT timer.
- **Example**:
  ```bash
  rustos# ping 10.0.2.2
  PING 10.0.2.2: 56 data bytes
  64 bytes from 10.0.2.2: icmp_seq=1 ttl=64 time=2 ms
  64 bytes from 10.0.2.2: icmp_seq=2 ttl=64 time=1 ms
  64 bytes from 10.0.2.2: icmp_seq=3 ttl=64 time=2 ms
  64 bytes from 10.0.2.2: icmp_seq=4 ttl=64 time=2 ms
  --- 10.0.2.2 ping statistics ---
  4 packets transmitted, 4 received, 0% packet loss, rtt avg = 1.7 ms
  ```

### 5. `httpget <url>`
- **Syntax**: `httpget http://10.0.2.2:8080/index.html`
- **Description**: Parses the URL into host, port, and path, connects via TCP 3-way handshake (SYN, SYN-ACK, ACK), sends an HTTP 1.1 GET request with `User-Agent: RustOS/1.0`, receives response segments, and outputs the HTTP status line, headers, and body.

### 6. `bc <expression>`
- **Syntax**: `bc <expr>`
- **Description**: Evaluates infix arithmetic expressions with operator precedence (`+`, `-`, `*`, `/`, `%`, `(`, `)`).
- **Example**:
  ```bash
  rustos# bc (15 + 5) * 4 / 2
  40
  ```

### 7. `research` / `deadlock`
- **Syntax**: `research` or `deadlock`
- **Description**: Launches the Deadlock Detection Research Lab graphical text UI.
