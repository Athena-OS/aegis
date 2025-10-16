pub const COMMON: &[&str] = &[
    "linux-firmware", "network-manager-applet", "man-db", "man-pages", "nano", "sudo", "curl",
    "accountsservice", "alacritty", "alsa-utils", "audit", "bind", "bluez", "dhcpcd", "dialog",
    "dosfstools", "irqbalance", "lvm2", "memtest86+", "most", "mtools", "nbd", "net-tools",
    "nfs-utils", "nss-mdns", "ntfsprogs", "pavucontrol", "pv", "rsync", "squashfs-tools",
    "syslinux", "testdisk", "usbutils", "wpa_supplicant", "xfsprogs",
    "pipewire", "pipewire-alsa", "wireplumber", "ntfs-3g", "zram-generator",
    "pocl", "asciinema", "bat", "bc", "bless", "cmatrix", "cowsay", "fastfetch", "file-roller",
    "fortune-mod", "git", "gparted", "gvfs-gphoto2", "gvfs-mtp", "hexedit", "jq", "keepassxc",
    "lolcat", "lsd", "nano-syntax-highlighting", "nautilus", "ncdu", "onionshare", "openvpn",
    "orca", "os-prober", "p7zip", "podman", "polkit", "sl", "torbrowser-launcher", "tree",
    "ufw", "unzip", "vim", "vnstat", "which", "xclip", "xmlstarlet", "zoxide",
    "athena-bash", "athena-config", "athena-kitty-config",
    "athena-tmux-config", "athena-tweak-tool", "athena-vscodium-themes", "athena-welcome",
    "htb-toolkit", "nist-feed",
];

pub const ARCH_ONLY: &[&str] = &[
    "systemd-sysvcompat","networkmanager","arch-install-scripts","broadcom-wl-dkms","edk2-shell",
    "grub","inetutils","mesa","mesa-utils","mkinitcpio-nfs-utils","mkinitcpio-openswap","netctl",
    "ntp","profile-sync-daemon","rtl8821cu-morrownr-dkms-git","sof-firmware","wireless_tools",
    "pipewire-pulse","pipewire-jack","ananicy","bashtop","imagemagick","lib32-glibc","mtpfs",
    "networkmanager-openvpn","octopi","openbsd-netcat","paru","pfetch","reflector","toilet-fonts",
    "wget","athena-cyber-hub","athena-firefox-config","athena-powershell-config","athena-vim-config",
    "kando-bin",
];

pub const FEDORA_ONLY: &[&str] = &[
    "NetworkManager","dnf5","e2fsprogs","alsa-sof-firmware","cracklib-dicts","grub2","iproute",
    "iputils","mesa-dri-drivers","mesa-vulkan-drivers","ntpsec","pciutils","selinux-policy-targeted",
    "NetworkManager-wifi","atheros-firmware","b43-fwcutter","b43-openfwwf","brcmfmac-firmware",
    "iwlegacy-firmware","iwlwifi-dvm-firmware","iwlwifi-mvm-firmware","libertas-firmware",
    "mt7xxx-firmware","nxpwireless-firmware","realtek-firmware","tiwilink-firmware","atmel-firmware",
    "zd1211-firmware","default-fonts","google-noto-fonts-common","google-noto-color-emoji-fonts",
    "google-noto-sans-cjk-fonts","pipewire-pulseaudio","pipewire-jack-audio-connection-kit",
    "btop","cronie","espeak","figlet","gtk-murrine-engine","ImageMagick","NetworkManager-openvpn",
    "netcat","nyancat","tidy","wget2-wget","kando","firefox-blackice",
];

// Small helpers
pub fn to_strings(slice: &[&str]) -> Vec<String> {
    slice.iter().map(|s| (*s).to_string()).collect()
}
pub fn extend(dst: &mut Vec<String>, slice: &[&str]) {
    dst.extend(slice.iter().copied().map(String::from));
}