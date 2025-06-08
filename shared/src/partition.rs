use crate::args;
use crate::args::PartitionMode;
use crate::exec::exec;
use crate::exec::exec_workdir;
use crate::files;
use crate::log::{debug, info};
use crate::returncode_eval::exec_eval;
use crate::returncode_eval::files_eval;
use crate::strings::crash;
use std::path::{Path, PathBuf};

/*mkfs.bfs mkfs.cramfs mkfs.ext3  mkfs.fat mkfs.msdos  mkfs.xfs
mkfs.btrfs mkfs.ext2  mkfs.ext4  mkfs.minix mkfs.vfat mkfs.f2fs */

fn extract_partition_number(bdevice: &str) -> String {
    bdevice
        .rfind(|c: char| !c.is_ascii_digit())
        .map(|pos| &bdevice[pos + 1..]) // Extract the digits
        .unwrap_or("") // Return empty string if no match
        .to_string() // Convert &str to String
}

fn encrypt_blockdevice(blockdevice: &str, cryptlabel: &str) {
    exec_eval(
        exec(
            "cryptsetup",
            vec![
                String::from("luksFormat"),
                String::from("-q"),
                String::from(blockdevice),
                String::from("-d"),
                String::from("/tmp/luks"),
            ],
        ),
        "Format LUKS partition",
    );
    exec_eval(
        exec(
            "cryptsetup",
            vec![
                String::from("luksOpen"),
                String::from(blockdevice),
                String::from(cryptlabel),
                String::from("-d"),
                String::from("/tmp/luks"),
            ],
        ),
        "Open LUKS format",
    );
    exec_eval(
        exec(
            "rm",
            vec![
                String::from("-rf"),
                String::from("/tmp/luks"),
            ],
        ),
        "Remove luks key",
    );
}

fn fmt_mount(diskdevice: &Path, partitiontype: &str, mountpoint: &str, filesystem: &str, blockdevice: &str, encryption: bool, is_efi: bool) {
    let mut bdevice = String::from(blockdevice);
    // Extract the block device name (i.e., sda3)
    let cryptlabel = format!("{}crypted",bdevice.trim_start_matches("/dev/")); // i.e., sda3crypted
    if encryption {
        encrypt_blockdevice(&bdevice, &cryptlabel);
        bdevice = format!("/dev/mapper/{cryptlabel}");
    }

    let mut needs_final_mount = true;

    match filesystem {
        "vfat" => exec_eval(
            exec("mkfs.vfat", vec![String::from("-F32"), String::from(&bdevice)]),
            format!("Formatting {bdevice} as vfat").as_str(),
        ),
        "bfs" => exec_eval(
            exec("mkfs.bfs", vec![String::from(&bdevice)]),
            format!("Formatting {bdevice} as bfs").as_str(),
        ),
        "cramfs" => exec_eval(
            exec("mkfs.cramfs", vec![String::from(&bdevice)]),
            format!("Formatting {bdevice} as cramfs").as_str(),
        ),
        "ext3" => exec_eval(
            exec("mkfs.ext3", vec![String::from("-F"), String::from(&bdevice)]),
            format!("Formatting {bdevice} as ext3").as_str(),
        ),
        "fat" => exec_eval(
            exec("mkfs.fat", vec![String::from(&bdevice)]),
            format!("Formatting {bdevice} as fat").as_str(),
        ),
        "msdos" => exec_eval(
            exec("mkfs.msdos", vec![String::from(&bdevice)]),
            format!("Formatting {bdevice} as msdos").as_str(),
        ),
        "xfs" => exec_eval(
            exec("mkfs.xfs", vec![String::from("-f"), String::from(&bdevice)]),
            format!("Formatting {bdevice} as xfs").as_str(),
        ),
        "btrfs" => {
            exec_eval(
                exec("mkfs.btrfs", vec![String::from("-f"), String::from(&bdevice)]),
                format!("Formatting {bdevice} as btrfs").as_str(),
            );
            mount(&bdevice, "/mnt", "");
            exec_eval(
                exec_workdir(
                    "btrfs",
                    "/mnt",
                    vec![
                        String::from("subvolume"),
                        String::from("create"),
                        String::from("@"),
                    ],
                ),
                "Create btrfs subvolume @",
            );
            exec_eval(
                exec_workdir(
                    "btrfs",
                    "/mnt",
                    vec![
                        String::from("subvolume"),
                        String::from("create"),
                        String::from("@home"),
                    ],
                ),
                "Create btrfs subvolume @home",
            );
            umount("/mnt");
            mount(&bdevice, "/mnt/", "subvol=@");
            files_eval(files::create_directory("/mnt/home"), "Create /mnt/home");
            mount(
                &bdevice,
                "/mnt/home",
                "subvol=@home",
            );
            needs_final_mount = false;
        }
        "ext2" => exec_eval(
            exec("mkfs.ext2", vec![String::from("-F"), String::from(&bdevice)]),
            format!("Formatting {bdevice} as ext2").as_str(),
        ),
        "ext4" => exec_eval(
            exec("mkfs.ext4", vec![String::from("-F"), String::from(&bdevice)]),
            format!("Formatting {bdevice} as ext4").as_str(),
        ),
        "minix" => exec_eval(
            exec("mkfs.minix", vec![String::from(&bdevice)]),
            format!("Formatting {bdevice} as minix").as_str(),
        ),
        "f2fs" => exec_eval(
            exec("mkfs.f2fs", vec![String::from("-f"), String::from(&bdevice)]),
            format!("Formatting {bdevice} as f2fs").as_str(),
        ),
        "linux-swap" => {
            exec_eval(
                exec("mkswap", vec![String::from("-L"), String::from("swap"), String::from(&bdevice)]),
                format!("Formatting {bdevice} as linux-swap").as_str(),
            );
            exec_eval(
                exec("swapon", vec![String::from(&bdevice)]),
                format!("Activate {bdevice} swap device").as_str(),
            );
            needs_final_mount = false;
        }
        "don't format" => {
            debug!("Not formatting {}", bdevice);
        }
        _ => {
            crash(
                format!("Unknown filesystem {filesystem}, used in partition {bdevice}"),
                1,
            );
        }
    }

    if partitiontype == "boot" {
        if is_efi {
        let esp_num = extract_partition_number(&bdevice);
            exec_eval(
                exec(
                    "parted",
                    vec![
                        String::from("-s"),
                        diskdevice.to_string_lossy().to_string(),
                        String::from("--"),
                        String::from("set"),
                        String::from(esp_num.as_str()), // It is the number ID of the EFI partition. i.e., if EFI partition is /dev/sda2, the number to set is 2
                        String::from("esp"),
                        String::from("on"),
                    ],
                ),
                format!("Enable EFI system partition on partition number {}", esp_num).as_str(),
            );
        }
        else {
            exec_eval(
                exec(
                    "parted",
                    vec![
                        String::from("-s"),
                        diskdevice.to_string_lossy().to_string(),
                        String::from("--"),
                        String::from("set"),
                        String::from("1"),
                        String::from("boot"),
                        String::from("on"),
                    ],
                ),
                "Set the root partition's boot flag to on",
            );
        }
    }

    if needs_final_mount {
        exec_eval(
            exec("mkdir", vec![String::from("-p"), String::from(mountpoint)]),
            format!("Creating mountpoint {mountpoint} for {bdevice}").as_str(),
        );
        mount(&bdevice, mountpoint, "");
    }
}

pub fn partition(
    device: PathBuf,
    mode: PartitionMode,
    encrypt_check: bool,
    efi: bool,
    swap: bool,
    swap_size: String,
    partitions: &mut Vec<args::Partition>,
) {
    if !device.exists() {
        crash(format!("The device {device:?} doesn't exist"), 1);
    }
    info!("{:?}", mode);
    match mode {
        PartitionMode::EraseDisk => {
            debug!("Erase disk partitioning {device:?}");
            if efi {
                partition_with_efi(&device, swap, swap_size, encrypt_check);
            } else {
                partition_no_efi(&device, swap, swap_size);
            }
            part_disk(&device, efi, encrypt_check, swap);
        }
        PartitionMode::Manual | PartitionMode::Replace => {
            debug!("Manual/Replace partitioning");
            partitions.sort_by(|a, b| a.mountpoint.len().cmp(&b.mountpoint.len()));
            for i in 0..partitions.len() {
                info!("{:?}", partitions);
                info!("{}", partitions.len());
                info!("Partition Type: {}", &partitions[i].partitiontype);
                info!("Mount point: {}", &partitions[i].mountpoint);
                info!("Filesystem: {}", &partitions[i].filesystem);
                info!("Block device: {}", &partitions[i].blockdevice);
                info!("To encrypt? {}", partitions[i].encrypt);
                fmt_mount(
                    &device,
                    &partitions[i].partitiontype,
                    &partitions[i].mountpoint,
                    &partitions[i].filesystem,
                    &partitions[i].blockdevice,
                    partitions[i].encrypt,
                    efi,
                );
            }
        }
    }
}

fn partition_with_efi(device: &Path, swap: bool, swap_size: String, encrypt_check: bool) {
    let device = device.to_string_lossy().to_string();
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("--"),
                String::from("mklabel"),
                String::from("gpt"),
            ],
        ),
        format!("Create gpt label on {}", &device).as_str(),
    );
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("--"),
                String::from("mkpart"),
                String::from("ESP"),
                String::from("fat32"),
                String::from("1MiB"),
                String::from("512MiB"),
            ],
        ),
        "Create EFI partition",
    );
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("--"),
                String::from("set"),
                String::from("1"), // It is the number ID of the EFI partition. It is 1 because we create it for first
                String::from("esp"),
                String::from("on"),
            ],
        ),
        "Enable EFI system partition",
    );

    let boundary_grub_partition_size = if encrypt_check {
        String::from("1536MiB") // 1024 MiB + 512 MiB
    } else {
        String::from("512MiB")
    };

    if encrypt_check {
        exec_eval(
            exec(
                "parted",
                vec![
                    String::from("-s"),
                    String::from(&device),
                    String::from("--"),
                    String::from("mkpart"),
                    String::from("primary"),
                    String::from("ext4"),
                    String::from("512MiB"),
                    String::from(&boundary_grub_partition_size),
                ],
            ),
            "Create grub boot partition",
        );
    }

    let boundary_partition_size = if swap {
        swap_size
    } else {
        String::from(&boundary_grub_partition_size)
    };

    if swap {
        exec_eval(
            exec(
                "parted",
                vec![
                    String::from("-s"),
                    String::from(&device),
                    String::from("--"),
                    String::from("mkpart"),
                    String::from("swap"),
                    String::from("linux-swap"),
                    String::from(&boundary_grub_partition_size),
                    String::from(&boundary_partition_size),
                ],
            ),
            "Create swap partition",
        );
    }

    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("--"),
                String::from("mkpart"),
                String::from("primary"),
                String::from("btrfs"),
                String::from(&boundary_partition_size),
                String::from("100%"),
            ],
        ),
        "Create btrfs root partition",
    );
}

fn partition_no_efi(device: &Path, swap: bool, swap_size: String) {
    let device = device.to_string_lossy().to_string();
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("--"),
                String::from("mklabel"),
                String::from("msdos"),
            ],
        ),
        format!("Create msdos label on {}", device).as_str(),
    );
    /* Create a dedicated legacy boot partition. Needed mostly in case of LUKS: https://bbs.archlinux.org/viewtopic.php?pid=2160947 */
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("--"),
                String::from("mkpart"),
                String::from("primary"),
                String::from("ext4"),
                String::from("1MiB"),
                String::from("512MiB"),
            ],
        ),
        "Create bios boot partition",
    );
    let boundary_partition_size = if swap {
        swap_size
    } else {
        String::from("512MiB")
    };

    if swap {
        exec_eval(
            exec(
                "parted",
                vec![
                    String::from("-s"),
                    String::from(&device),
                    String::from("--"),
                    String::from("mkpart"),
                    String::from("primary"),
                    String::from("linux-swap"),
                    String::from("512MiB"),
                    String::from(&boundary_partition_size),
                ],
            ),
            "Create swap partition",
        );
    }

    /* Root Partition. Created as last partition to allow users to shrink or extend*/
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("--"),
                String::from("mkpart"),
                String::from("primary"),
                String::from("btrfs"),
                String::from(&boundary_partition_size),
                String::from("100%"),
            ],
        ),
        "Create btrfs root partition",
    );
    // The following is needed because boot GRUB partition is inside the 'device' disk
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("--"),
                String::from("set"),
                String::from("1"),
                String::from("boot"),
                String::from("on"),
            ],
        ),
        "Set the root partition's boot flag to on",
    );
}

fn part_disk(device: &Path, efi: bool, encrypt_check: bool, swap: bool) {
    let device = device.to_string_lossy().to_string(); // i.e., /dev/sda

    let dsuffix = if device.contains("nvme") || device.contains("mmcblk") || device.contains("loop") {
        "p"
    }
    else {
        ""
    };

    let mut dindex = 1;

    if efi {
        /* Format EFI partition */
        exec_eval(
            exec(
                "mkfs.fat",
                vec![String::from("-F"), String::from("32"), String::from("-n"), String::from("BOOT"), format!("{}{}{}", device, dsuffix, dindex)],
            ),
            format!("Format {}{}{} as fat32", device, dsuffix, dindex).as_str(),
        );

        if encrypt_check {
            dindex += 1; // Next partition index
            exec_eval(
                exec("mkfs.ext4", vec![String::from("-F"), format!("{}{}{}", device, dsuffix, dindex)]),
                format!("Format {}{}{} as ext4", device, dsuffix, dindex).as_str(),
            );
        }

    } else {
        /* Format GRUB Legacy partition */
        exec_eval(
            exec("mkfs.ext4", vec![String::from("-F"), format!("{}{}{}", device, dsuffix, dindex)]),
            format!("Format {}{}{} as ext4", device, dsuffix, dindex).as_str(),
        );
    }

    /* Format Swap partition */
    if swap {
        dindex += 1; // Next partition index
        exec_eval(
            exec(
                "mkswap",
                vec![String::from("-L"), String::from("swap"), format!("{}{}{}", device, dsuffix, dindex)],
            ),
            format!("Make {}{}{} as swap partition", device, dsuffix, dindex).as_str(),
        );
        exec_eval(
            exec(
                "swapon",
                vec![format!("{}{}{}", device, dsuffix, dindex)],
            ),
            format!("Activate {}{}{} swap device", device, dsuffix, dindex).as_str(),
        );
    }
    dindex += 1; // Next partition index
    let mut root_blockdevice = format!("{device}{dsuffix}{dindex}"); // i.e., /dev/sda3
    let root_blockdevice_name = root_blockdevice.trim_start_matches("/dev/"); // i.e., sda3

    if encrypt_check {
        let cryptlabel = format!("{root_blockdevice_name}crypted"); // i.e., sda3crypted will be the name of the opened LUKS partition
        encrypt_blockdevice(&root_blockdevice, &cryptlabel);
        root_blockdevice =  format!("/dev/mapper/{cryptlabel}");
    }

    /* Format root partition */
    exec_eval(
        exec(
            "mkfs.btrfs",
            vec![String::from("-L"), String::from("athenaos"), String::from("-f"), format!("{}", root_blockdevice)],
        ),
        format!("Format {} as btrfs", root_blockdevice).as_str(),
    );
    mount(&root_blockdevice, "/mnt", "");
    exec_eval(
        exec_workdir(
            "btrfs",
            "/mnt",
            vec![
                String::from("subvolume"),
                String::from("create"),
                String::from("@"),
            ],
        ),
        "Create btrfs subvolume @",
    );
    exec_eval(
        exec_workdir(
            "btrfs",
            "/mnt",
            vec![
                String::from("subvolume"),
                String::from("create"),
                String::from("@home"),
            ],
        ),
        "Create btrfs subvolume @home",
    );
    umount("/mnt");
    mount(&root_blockdevice, "/mnt/", "subvol=@");
    let mount_path = if efi {
        "/mnt/boot/efi".to_string()
    } else {
        "/mnt/boot".to_string()
    };
    files_eval(files::create_directory(&mount_path), &format!("Create {}", mount_path));
    files_eval(files::create_directory("/mnt/home"), "Create /mnt/home");
    mount(
        &root_blockdevice,
        "/mnt/home",
        "subvol=@home",
    );

    if efi && encrypt_check {
        mount(format!("{}{}2", device, dsuffix).as_str(), "/mnt/boot", "");
    }
    mount(format!("{}{}1", device, dsuffix).as_str(), &mount_path, "");
}

pub fn mount(partition: &str, mountpoint: &str, options: &str) {
    if !options.is_empty() {
        exec_eval(
            exec(
                "mount",
                vec![
                    String::from(partition),
                    String::from(mountpoint),
                    String::from("-o"),
                    String::from(options),
                ],
            ),
            format!(
                "Mount {} with options {} at {}",
                partition, options, mountpoint
            )
            .as_str(),
        );
    } else {
        exec_eval(
            exec(
                "mount",
                vec![String::from(partition), String::from(mountpoint)],
            ),
            format!("Mount {} with no options at {}", partition, mountpoint).as_str(),
        );
    }
}

pub fn umount(mountpoint: &str) {
    exec_eval(
        exec("umount", vec![String::from("-R"), String::from(mountpoint)]),
        format!("Unmount command processed on {}", mountpoint).as_str(),
    );
}
