use crate::args;
use crate::args::PartitionMode;
use crate::exec::exec;
use crate::exec::exec_workdir;
use crate::files;
use crate::log::debug;
use crate::returncode_eval::exec_eval;
use crate::returncode_eval::files_eval;
use crate::strings::crash;
use std::path::{Path, PathBuf};

/*mkfs.bfs mkfs.cramfs mkfs.ext3  mkfs.fat mkfs.msdos  mkfs.xfs
mkfs.btrfs mkfs.ext2  mkfs.ext4  mkfs.minix mkfs.vfat mkfs.f2fs */

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

pub fn fmt_mount(mountpoint: &str, filesystem: &str, blockdevice: &str, encryption: bool) {
    let mut bdevice = String::from(blockdevice);
    // Extract the block device name (i.e., sda3)
    let cryptlabel = format!("{}crypted",bdevice.trim_start_matches("/dev/")); // i.e., sda3crypted
    if encryption {
        encrypt_blockdevice(&bdevice, &cryptlabel);
        bdevice = format!("/dev/mapper/{cryptlabel}");
    }
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
        }
        "don't format" => {
            debug!("Not formatting {}", bdevice);
        }
        "noformat" => {
            debug!("Not formatting {}", bdevice);
        }
        _ => {
            crash(
                format!("Unknown filesystem {filesystem}, used in partition {bdevice}"),
                1,
            );
        }
    }
    exec_eval(
        exec("mkdir", vec![String::from("-p"), String::from(mountpoint)]),
        format!("Creating mountpoint {mountpoint} for {bdevice}").as_str(),
    );
    mount(&bdevice, mountpoint, "");
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
    println!("{:?}", mode);
    match mode {
        PartitionMode::Auto => {
            if !device.exists() {
                crash(format!("The device {device:?} doesn't exist"), 1);
            }
            debug!("automatically partitioning {device:?}");
            if efi {
                partition_with_efi(&device, swap, swap_size);
            } else {
                partition_no_efi(&device, swap, swap_size);
            }
            part_disk(&device, efi, encrypt_check, swap);
        }
        PartitionMode::Manual | PartitionMode::Replace => {
            debug!("Manual/Replace partitioning");
            partitions.sort_by(|a, b| a.mountpoint.len().cmp(&b.mountpoint.len()));
            for i in 0..partitions.len() {
                println!("{:?}", partitions);
                println!("{}", partitions.len());
                println!("{}", &partitions[i].mountpoint);
                println!("{}", &partitions[i].filesystem);
                println!("{}", &partitions[i].blockdevice);
                println!("{}", partitions[i].encrypt);
                fmt_mount(
                    &partitions[i].mountpoint,
                    &partitions[i].filesystem,
                    &partitions[i].blockdevice,
                    partitions[i].encrypt,
                );
            }
        }
    }
}

fn partition_with_efi(device: &Path, swap: bool, swap_size: String) {
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
        format!("create gpt label on {}", &device).as_str(),
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
        "create EFI partition",
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
        "enable EFI system partition",
    );
    let boundary_partition_size = if swap {
        format!("-{}", swap_size)
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
                    String::from("swap"),
                    String::from("linux-swap"),
                    String::from("512MiB"),
                    String::from(&boundary_partition_size),
                ],
            ),
            "create swap partition",
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
        "create btrfs root partition",
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
        "create bios boot partition",
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
            "create swap partition",
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
        "create btrfs root partition",
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
        "set the root partition's boot flag to on",
    );
}

fn part_disk(device: &Path, efi: bool, encrypt_check: bool, swap: bool) {
    let device = device.to_string_lossy().to_string(); // i.e., /dev/sda

    let dsuffix = if device.contains("nvme") || device.contains("mmcblk") {
        "p"
    }
    else {
        ""
    };

    let bdsuffix = if swap {
        format!("{}3", dsuffix)
    } else {
        format!("{}2", dsuffix)
    };

    if efi {
        /* Format EFI partition */
        exec_eval(
            exec(
                "mkfs.fat",
                vec![String::from("-F"), String::from("32"), String::from("-n"), String::from("BOOT"), format!("{}{}1", device, dsuffix)],
            ),
            format!("format {}{}1 as fat32", device, dsuffix).as_str(),
        );
    } else if !efi {
        /* Format GRUB Legacy partition */
        exec_eval(
            exec("mkfs.ext4", vec![String::from("-F"), format!("{}{}1", device, dsuffix)]),
            format!("format {}{}1 as ext4", device, dsuffix).as_str(),
        );
    }

    /* Format Swap partition */
    if swap {
        exec_eval(
            exec(
                "mkswap",
                vec![String::from("-L"), String::from("swap"), format!("{}{}2", device, dsuffix)],
            ),
            format!("make {}{}2 as swap partition", device, dsuffix).as_str(),
        );
        exec_eval(
            exec(
                "swapon",
                vec![format!("{}{}2", device, dsuffix)],
            ),
            format!("activate {}{}2 swap device", device, dsuffix).as_str(),
        );
    }

    let mut root_blockdevice = format!("{device}{bdsuffix}"); // i.e., /dev/sda3
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
        format!("format {} as btrfs", root_blockdevice).as_str(),
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
    files_eval(files::create_directory("/mnt/boot"), "create /mnt/boot");
    files_eval(files::create_directory("/mnt/home"), "create /mnt/home");
    mount(
        &root_blockdevice,
        "/mnt/home",
        "subvol=@home",
    );

    mount(format!("{}{}1", device, dsuffix).as_str(), "/mnt/boot", "");
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
                "mount {} with options {} at {}",
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
            format!("mount {} with no options at {}", partition, mountpoint).as_str(),
        );
    }
}

pub fn umount(mountpoint: &str) {
    exec_eval(
        exec("umount", vec![String::from("-R"), String::from(mountpoint)]),
        format!("unmount command processed on {}", mountpoint).as_str(),
    );
}
