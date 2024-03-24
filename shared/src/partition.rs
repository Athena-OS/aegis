use crate::args;
use crate::args::PartitionMode;
use crate::exec::exec;
use crate::exec::exec_workdir;
use crate::files;
use crate::log::debug;
use crate::returncode_eval::exec_eval;
use crate::returncode_eval::files_eval;
use crate::strings::crash;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;

fn encrypt_blockdevice(blockdevice: &str, cryptlabel: &str) {
    // LUKS formatting
    let lookupform = Command::new("secret-tool")
        .arg("lookup")
        .arg("luks-key")
        .arg(cryptlabel)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to execute secret-tool");

    Command::new("cryptsetup")
        .arg("luksFormat")
        .arg(blockdevice)
        .arg(String::from("-"))
        .stdin(lookupform.stdout.unwrap())
        .output()
        .expect("Failed to execute cryptsetup");
    
    // LUKS opening
    let lookupopen = Command::new("secret-tool")
        .arg("lookup")
        .arg("luks-key")
        .arg(cryptlabel)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to execute secret-tool");

    Command::new("cryptsetup")
        .arg("luksOpen")
        .arg(blockdevice)
        .arg(cryptlabel)
        .arg(String::from("-"))
        .stdin(lookupopen.stdout.unwrap())
        .output()
        .expect("Failed to execute cryptsetup");
}

/*mkfs.bfs mkfs.cramfs mkfs.ext3  mkfs.fat mkfs.msdos  mkfs.xfs
mkfs.btrfs mkfs.ext2  mkfs.ext4  mkfs.minix mkfs.vfat mkfs.f2fs */

pub fn fmt_mount(mountpoint: &str, filesystem: &str, blockdevice: &str, encryption: bool) {
    let mut device = String::from(blockdevice);
    // Extract the block device name
    let re = Regex::new(r"^/dev/(\w+)").unwrap();
    let cryptlabel = re
        .captures(&device)
        .and_then(|c| c.get(1))
        .map(|bd| bd.as_str().to_string())
        .unwrap_or_default();
    if encryption {
        encrypt_blockdevice(&device, &cryptlabel);
        device = format!("/dev/mapper/{cryptlabel}");
    }
    match filesystem {
        "vfat" => exec_eval(
            exec("mkfs.vfat", vec![String::from("-F32"), String::from(&device)]),
            format!("Formatting {device} as vfat").as_str(),
        ),
        "bfs" => exec_eval(
            exec("mkfs.bfs", vec![String::from(&device)]),
            format!("Formatting {device} as bfs").as_str(),
        ),
        "cramfs" => exec_eval(
            exec("mkfs.cramfs", vec![String::from(&device)]),
            format!("Formatting {device} as cramfs").as_str(),
        ),
        "ext3" => exec_eval(
            exec("mkfs.ext3", vec![String::from(&device)]),
            format!("Formatting {device} as ext3").as_str(),
        ),
        "fat" => exec_eval(
            exec("mkfs.fat", vec![String::from(&device)]),
            format!("Formatting {device} as fat").as_str(),
        ),
        "msdos" => exec_eval(
            exec("mkfs.msdos", vec![String::from(&device)]),
            format!("Formatting {device} as msdos").as_str(),
        ),
        "xfs" => exec_eval(
            exec("mkfs.xfs", vec![String::from(&device)]),
            format!("Formatting {device} as xfs").as_str(),
        ),
        "btrfs" => {
            exec_eval(
                exec("mkfs.btrfs", vec![String::from("-f"), String::from(&device)]),
                format!("Formatting {device} as btrfs").as_str(),
            );
        }
        "ext2" => exec_eval(
            exec("mkfs.ext2", vec![String::from(&device)]),
            format!("Formatting {device} as ext2").as_str(),
        ),
        "ext4" => exec_eval(
            exec("mkfs.ext4", vec![String::from(&device)]),
            format!("Formatting {device} as ext4").as_str(),
        ),
        "minix" => exec_eval(
            exec("mkfs.minix", vec![String::from(&device)]),
            format!("Formatting {device} as minix").as_str(),
        ),
        "f2fs" => exec_eval(
            exec("mkfs.f2fs", vec![String::from(&device)]),
            format!("Formatting {device} as f2fs").as_str(),
        ),
        "linux-swap" => {
            exec_eval(
                exec("mkswap", vec![String::from(&device)]),
                format!("Formatting {device} as linux-swap").as_str(),
            );
            exec_eval(
                exec("swapon", vec![String::from(&device)]),
                format!("Activate {device} swap device").as_str(),
            );
        }
        "don't format" => {
            debug!("Not formatting {}", device);
        }
        "noformat" => {
            debug!("Not formatting {}", device);
        }
        _ => {
            crash(
                format!("Unknown filesystem {filesystem}, used in partition {device}"),
                1,
            );
        }
    }
    exec_eval(
        exec("mkdir", vec![String::from("-p"), String::from(mountpoint)]),
        format!("Creating mountpoint {mountpoint} for {device}").as_str(),
    );
    mount(&device, mountpoint, "");
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
            if device.to_string_lossy().contains("nvme")
                || device.to_string_lossy().contains("mmcblk")
            {
                part_nvme(&device, efi, encrypt_check, swap);
            } else {
                part_disk(&device, efi, encrypt_check, swap);
            }
        }
        PartitionMode::Manual => {
            debug!("Manual partitioning");
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
    // No need to create ext4 GRUB partition because MBR should automatically create it inside in the boot sector
    /*exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("--"),
                String::from("mkpart"),
                String::from("primary"),
                String::from("ext4"),
                String::from("1MIB"),
                String::from("512MIB"),
            ],
        ),
        "create bios boot partition",
    );*/
    let boundary_partition_size = if swap {
        format!("-{}", swap_size)
    } else {
        String::from("100%")
    };
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
                String::from("1MIB"), // 1MiB instead of 512MiB because we removed the explicit creation of ext4 boot partition for bios-legacy case
                String::from(&boundary_partition_size),
            ],
        ),
        "create btrfs root partition",
    );
    // The following is needed because boot GRUB partition is not created explicitely but automatically created by MBR in the boot sector
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
                    String::from(&boundary_partition_size),
                    String::from("100%"),
                ],
            ),
            "create swap partition",
        );
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
                String::from("512MIB"),
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
        String::from("100%")
    };
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
                String::from("512MIB"),
                String::from(&boundary_partition_size),
            ],
        ),
        "create btrfs root partition",
    );
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
                    String::from(&boundary_partition_size),
                    String::from("100%"),
                ],
            ),
            "create swap partition",
        );
    }
}

fn part_nvme(device: &Path, efi: bool, encrypt_check: bool, swap: bool) {
    let device = device.to_string_lossy().to_string();
    let mut bdevice = device.clone();

    if efi {
        if encrypt_check {
            encrypt_blockdevice(format!("{bdevice}p2").as_str(), "auto"); // auto is the attr value of secret-tool defined in Aegis TUI and GUI for Auto mode
            bdevice = String::from("/dev/mapper/auto");
        }
        exec_eval(
            exec(
                "mkfs.fat",
                vec![String::from("-F"), String::from("32"), String::from("-n"), String::from("boot"), format!("{}p1", device)],
            ),
            format!("format {}p1 as fat32", device).as_str(),
        );

        exec_eval(
            exec(
                "mkfs.btrfs",
                vec!["-L".to_string(), "athenaos".to_string(), "-f".to_string(), format!("{}p2", bdevice)],
            ),
            format!("format {}p2 as btrfs", bdevice).as_str(),
        );
        mount(format!("{}p2", bdevice).as_str(), "/mnt", "");
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
        mount(format!("{}p2", bdevice).as_str(), "/mnt/", "subvol=@");
        files_eval(files::create_directory("/mnt/boot"), "create /mnt/boot");
        files_eval(files::create_directory("/mnt/home"), "create /mnt/home");
        mount(
            format!("{}p2", bdevice).as_str(),
            "/mnt/home",
            "subvol=@home",
        );

        mount(format!("{}p1", device).as_str(), "/mnt/boot", "");

        if swap {
            exec_eval(
                exec(
                    "mkswap",
                    vec!["-L".to_string(), "swap".to_string(), format!("{}p3", device)],
                ),
                format!("make {}p3 as swap partition", device).as_str(),
            );
            exec_eval(
                exec(
                    "swapon",
                    vec![format!("{}p3", device)],
                ),
                format!("activate {}p3 swap device", device).as_str(),
            );
        }
    } else if !efi{
        if encrypt_check {
            encrypt_blockdevice(format!("{bdevice}p1").as_str(), "auto"); // auto is the attr value of secret-tool defined in Aegis TUI and GUI for Auto mode
            bdevice = String::from("/dev/mapper/auto");
        }
        // No need to create ext4 GRUB partition because MBR should automatically create it inside the boot sector
        /*exec_eval(
            exec("mkfs.ext4", vec![format!("{}p1", device)]),
            format!("format {}p1 as ext4", device).as_str(),
        );*/
        exec_eval(
            exec(
                "mkfs.btrfs",
                vec!["-L".to_string(), "athenaos".to_string(), "-f".to_string(), format!("{}p1", bdevice)],
            ),
            format!("format {}p1 as btrfs", bdevice).as_str(),
        );
        mount(format!("{}p1", bdevice).as_str(), "/mnt/", "");
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
        mount(format!("{}p1", bdevice).as_str(), "/mnt/", "subvol=@");
        files_eval(files::create_directory("/mnt/boot"), "create /mnt/boot");
        files_eval(files::create_directory("/mnt/home"), "create /mnt/home");
        mount(
            format!("{}p1", bdevice).as_str(),
            "/mnt/home",
            "subvol=@home",
        );

        // No need to create ext4 GRUB partition because MBR should automatically create it inside the boot sector
        //mount(format!("{}p1", device).as_str(), "/mnt/boot", "");

        if swap {
            exec_eval(
                exec(
                    "mkswap",
                    vec!["-L".to_string(), "swap".to_string(), format!("{}p2", device)],
                ),
                format!("make {}p2 as swap partition", device).as_str(),
            );
            exec_eval(
                exec(
                    "swapon",
                    vec![format!("{}p2", device)],
                ),
                format!("activate {}p2 swap device", device).as_str(),
            );
        }
    }
}

fn part_disk(device: &Path, efi: bool, encrypt_check: bool, swap: bool) {
    let device = device.to_string_lossy().to_string();
    let mut bdevice = device.clone();

    if efi {
        if encrypt_check {
            encrypt_blockdevice(format!("{bdevice}2").as_str(), "auto"); // auto is the attr value of secret-tool defined in Aegis TUI and GUI for Auto mode
            bdevice = String::from("/dev/mapper/auto");
        }
        exec_eval(
            exec(
                "mkfs.fat",
                vec![String::from("-F"), String::from("32"), String::from("-n"), String::from("boot"), format!("{}1", device)],
            ),
            format!("format {}1 as fat32", device).as_str(),
        );

        exec_eval(
            exec("mkfs.btrfs", vec!["-L".to_string(), "athenaos".to_string(), "-f".to_string(), format!("{}2", bdevice)]),
            format!("format {}2 as btrfs", bdevice).as_str(),
        );
        mount(format!("{}2", bdevice).as_str(), "/mnt", "");
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
        mount(format!("{}2", bdevice).as_str(), "/mnt/", "subvol=@");
        files_eval(files::create_directory("/mnt/boot"), "create /mnt/boot");
        files_eval(files::create_directory("/mnt/home"), "create /mnt/home");
        mount(format!("{}2", bdevice).as_str(), "/mnt/home", "subvol=@home");

        mount(format!("{}1", device).as_str(), "/mnt/boot", "");

        if swap {
            exec_eval(
                exec(
                    "mkswap",
                    vec!["-L".to_string(), "swap".to_string(), format!("{}3", device)],
                ),
                format!("make {}3 as swap partition", device).as_str(),
            );
            exec_eval(
                exec(
                    "swapon",
                    vec![format!("{}3", device)],
                ),
                format!("activate {}3 swap device", device).as_str(),
            );
        }
    } else if !efi {
        if encrypt_check {
            encrypt_blockdevice(format!("{bdevice}1").as_str(), "auto"); // auto is the attr value of secret-tool defined in Aegis TUI and GUI for Auto mode
            bdevice = String::from("/dev/mapper/auto");
        }
        // No need to create ext4 GRUB partition because MBR should automatically create it inside the boot sector
        /*exec_eval(
            exec("mkfs.ext4", vec![format!("{}1", device)]),
            format!("format {}1 as ext4", device).as_str(),
        );*/
        exec_eval(
            exec("mkfs.btrfs", vec!["-L".to_string(), "athenaos".to_string(), "-f".to_string(), format!("{}1", bdevice)]),
            format!("format {}1 as btrfs", bdevice).as_str(),
        );
        mount(format!("{}1", bdevice).as_str(), "/mnt/", "");
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
            "create btrfs subvolume @home",
        );
        umount("/mnt");
        mount(format!("{}1", bdevice).as_str(), "/mnt/", "subvol=@");
        files_eval(
            files::create_directory("/mnt/boot"),
            "create directory /mnt/boot",
        );
        files_eval(
            files::create_directory("/mnt/home"),
            "create directory /mnt/home",
        );
        mount(format!("{}1", bdevice).as_str(), "/mnt/home", "subvol=@home");

        // No need to create ext4 GRUB partition because MBR should automatically create it inside the boot sector
        //mount(format!("{}1", device).as_str(), "/mnt/boot", "");

        if swap {
            exec_eval(
                exec(
                    "mkswap",
                    vec!["-L".to_string(), "swap".to_string(), format!("{}2", device)],
                ),
                format!("make {}2 as swap partition", device).as_str(),
            );
            exec_eval(
                exec(
                    "swapon",
                    vec![format!("{}2", device)],
                ),
                format!("activate {}2 swap device", device).as_str(),
            );
        }
    }
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
        exec("umount", vec![String::from(mountpoint)]),
        format!("unmount command processed on {}", mountpoint).as_str(),
    );
}
