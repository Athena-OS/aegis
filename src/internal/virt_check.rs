use std::process::Command;
use std::io::Read;
use crate::args::PackageManager;
use crate::internal::*;

pub fn virt_check() {
    let output = Command::new("systemd-detect-virt")
        .output()
        .expect("Failed to run systemd-detect-virt");

    let result = String::from_utf8_lossy(&output.stdout);

    if result == "oracle" {
        install(PackageManager::Pacman, vec!["virtualbox-guest-utils"]);
        enable_service("vboxservice");
    } else if result == "vmware" {
        install(PackageManager::Pacman, vec!["open-vm-tools", "xf86-video-vmware"]);
        enable_service("vmware-vmblock-fuse");
        enable_service("vmtoolsd");
        enable_service("mnt-hgfs.mount");

        exec_eval(
            exec_chroot(
                "sed",
                vec![
                    String::from("-in"),
                    String::from("'/^MODULES*/ s/"$/ vsock vmw_vsock_vmci_transport vmw_balloon vmw_vmci vmwgfx"/g'"),
                    String::from("/etc/mkinitcpio.conf"), //In chroot we don't need to specify /mnt
                ],
            ),
            "Set vmware kernel parameter",
        );
    } else if result == "qemu" || result == "kvm" {
        install(PackageManager::Pacman, vec!["qemu-guest-agent"]);
        enable_service("qemu-guest-agent");
    } else if result == "microsoft" {
        install(PackageManager::Pacman, vec!["hyperv", "xf86-video-fbdev"]);
        enable_service("hv_fcopy_daemon");
        enable_service("hv_kvp_daemon");
        enable_service("hv_vss_daemon");

        exec_eval(
            exec_chroot(
                "sed",
                vec![
                    String::from("-in"),
                    String::from("'/^GRUB_CMDLINE_LINUX_DEFAULT*/ s/"$/ video=hyperv_fb:3840x2160"/g'"),
                    String::from("/etc/default/grub"), //In chroot we don't need to specify /mnt
                ],
            ),
            "Set hyperv kernel parameter",
        );
    }
}