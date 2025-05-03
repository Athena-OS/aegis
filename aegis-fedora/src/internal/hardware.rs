use shared::info;
use shared::exec::exec_output;
use shared::returncode_eval::exec_eval_result;

type Packages = Vec<&'static str>;
type Services = Vec<&'static str>;
type SetParams = Vec<(String, Vec<String>)>;

pub fn virt_check() -> (Packages, Services, SetParams) {
    let output = exec_eval_result(
        exec_output(
            "systemd-detect-virt",
            vec![]
        ),
        "Detect the virtualization environment",
    );

    let mut result = String::from_utf8_lossy(&output.stdout).to_string();
    result.pop(); //Removing the \n char from string

    let mut packages = Vec::new();
    let mut services = Vec::new();
    let mut set_params = Vec::new(); // To store the commands for file changes by sed

    if result == "oracle" {
        packages.push("virtualbox-guest-additions");
        services.push("vboxservice");
    } else if result == "vmware" {
        packages.extend(vec!["open-vm-tools", "xorg-x11-drv-vmware"]);
        services.extend(vec!["vmtoolsd"]);
    
    } else if result == "qemu" || result == "kvm" {
        packages.extend(vec!["qemu-guest-agent", "spice-vdagent"]);
        services.push("qemu-guest-agent");
    } else if result == "microsoft" {
        packages.extend(vec!["hyperv-tools"]);

        // Add the file change for Hyper-V kernel parameter
        set_params.push((
            "Set hyperv kernel parameter".to_string(),
            vec![
                "-i".to_string(),
                "-e".to_string(),
                "/^GRUB_CMDLINE_LINUX_DEFAULT*/ s/\"$/ video=hyperv_fb:3840x2160\"/g".to_string(),
                "/mnt/etc/default/grub".to_string(),
            ],
        ));
    }

    (packages, services, set_params) // Return packages, services, and file changes
}

pub fn cpu_gpu_check() -> Vec<&'static str> {
    let mut packages = Vec::new();

    // Detect CPU
    if cpu_detect().contains("Intel") {
        info!("Intel CPU detected.");
        packages.extend(vec!["intel-compute-runtime"]);
    } else if cpu_detect().contains("AMD") {
        info!("AMD CPU detected.");
        packages.push("amd-ucode-firmware");
    }

    // Detect GPU
    let gpudetect_output = exec_eval_result(
        exec_output(
            "lspci",
            vec![
                String::from("-k")
            ]
        ),
        "Detect the GPU",
    );

    let gpudetect = String::from_utf8_lossy(&gpudetect_output.stdout);

    if gpudetect.contains("AMD") {
        info!("AMD GPU detected.");
        packages.extend(vec!["xorg-x11-drv-amdgpu", "amd-gpu-firmware"]);
    }

    if gpudetect.contains("ATI") && !gpudetect.contains("AMD") {
        info!("ATI GPU detected.");
        packages.push("mesa-libOpenCL");
    }

    if gpudetect.contains("NVIDIA") {
        info!("NVIDIA GPU detected.");
        packages.push("nvidia-gpu-firmware");

        packages.extend(vec!["gwe", "nvtop"]);

        // Add envycontrol if hybrid GPU setup detected
        /*
        if gpudetect.contains("Intel") || gpudetect.contains("AMD") || gpudetect.contains("ATI") {
            packages.push("envycontrol");
        }
        */
    }

    packages // Return the list of packages
}

fn cpu_detect() -> String {
    let lscpu_output = exec_eval_result(
        exec_output(
            "lscpu",
            vec![]
        ),
        "Detect the CPU",
    );

    let lscpu_str = std::str::from_utf8(&lscpu_output.stdout)
        .expect("Failed to parse lscpu output as UTF-8");

    let vendor_id_line = lscpu_str
        .lines()
        .find(|line| line.starts_with("Vendor ID:"))
        .expect("Vendor ID not found in lscpu output");

    let vendor_id = vendor_id_line
        .split(':')
        .nth(1)
        .expect("Invalid format for Vendor ID in lscpu output")
        .trim();

    vendor_id.to_string()
}