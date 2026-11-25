# Oracle VirtualBox Setup & Execution Guide

## 1. Prerequisites
- **Host OS**: Windows 11 (64-bit).
- **Virtualization Software**: **Oracle VM VirtualBox** (Version 7.0 or 6.1).
- **Deliverable**: `deliverables/rustos.iso` (Pre-built bootable ISO image).

> [!IMPORTANT]
> **No QEMU**: This project is exclusively targeted, tested, and validated for Oracle VirtualBox.

---

## 2. Automated VM Setup & Launch Script

RustOS provides automated launch scripts that configure the VirtualBox VM with exact hardware settings:

### Windows PowerShell:
```powershell
.\launch_vbox.ps1
```

### Windows Command Prompt / Batch:
```cmd
launch_vbox.bat
```

The script automatically executes `VBoxManage` to:
1. Create a VM named `RustOS`.
2. Configure OS type as `Other/Unknown` (32-bit).
3. Set RAM to `128 MB` and VRAM to `16 MB`.
4. Create an IDE storage controller and attach `deliverables/rustos.iso`.
5. Configure Network Adapter 1 as **Intel PRO/1000 MT Desktop (82540EM)** in **NAT mode**.
6. Set up COM1 serial port redirected to `rustos_serial.log` for kernel debugging.
7. Start the VM.

---

## 3. Manual VirtualBox VM Setup (GUI Step-by-Step)

If you prefer to configure the VM manually in the VirtualBox GUI:

### Step 1: Create Virtual Machine
1. Open **Oracle VM VirtualBox Manager**.
2. Click **Machine -> New** (or press `Ctrl+N`).
3. Set the following fields:
   - **Name**: `RustOS`
   - **Type**: `Other`
   - **Version**: `Other/Unknown (32-bit)`
4. Memory size: Set Base Memory to **128 MB** (or 256 MB).
5. Processors: **1 CPU**.
6. Hard disk: Select **"Do not add a virtual hard disk"** (RustOS runs entirely from the live CD ISO with in-memory RAMFS).
7. Click **Finish**.

### Step 2: Configure Storage
1. Select `RustOS` and click **Settings -> Storage**.
2. Under **Storage Devices**, select the empty Optical Drive icon under the IDE Controller.
3. Click the disk icon on the right and choose **"Choose a disk file..."**.
4. Browse and select `deliverables\rustos.iso`.
5. Check the box **"Live CD/DVD"**.

### Step 3: Configure Networking (Intel 82540EM)
1. In Settings, select the **Network** tab.
2. Ensure **Adapter 1** is **Enabled**.
3. **Attached to**: Choose **NAT** (or **Bridged Adapter** to join your physical LAN).
4. Expand **Advanced**:
   - **Adapter Type**: Select **Intel PRO/1000 MT Desktop (82540EM)**.
   - **Promiscuous Mode**: Allow All.
   - **Cable Connected**: Checked.

### Step 4: Configure Serial Port (Optional Debug Logging)
1. Select the **Serial Ports** tab.
2. Check **Enable Serial Port**.
3. **Port Number**: `COM1`, **Port Mode**: `Raw File`, **Path/Address**: `<path>\rustos_serial.log`.

### Step 5: Start the VM
1. Select `RustOS` in VirtualBox and click **Start**.
2. The VM will boot into the El Torito bootloader, initialize protected mode, and present the **RustOS Desktop**.

---

## 4. Interacting with RustOS

### Desktop Navigation:
- **Up / Down Arrow Keys**: Navigate the 7 main menu applications:
  1. `Shell`
  2. `Web Browser`
  3. `Network Settings`
  4. `File Manager`
  5. `Deadlock Detection Research Lab`
  6. `System Information`
  7. `About RustOS`
- **Enter**: Launch the highlighted application.
- **Esc**: Exit any application and return to the main Desktop.

### Capturing & Releasing the Keyboard:
- VirtualBox captures keyboard input when you click inside the VM window.
- To release the keyboard back to Windows, press the **Right Control** key (Host Key).
