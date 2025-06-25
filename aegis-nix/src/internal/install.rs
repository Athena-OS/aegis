    use shared::{error, info};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn install(cores: String, jobs: String) -> i32 {
    // The init logging is called at the beginning of main.rs

    let install_nixos_args = format!("nixos-install --no-root-password --cores {} --max-jobs {} --keep-going", cores, jobs);
    let install_args = vec![
        "-p",
        "nixos-install-tools",
        "--run",
        &install_nixos_args,
    ];
    // nix-shell seems to work as non-sudo only by using --run; --command works only as sudo
    let mut install_cmd = Command::new("nix-shell")
        .args(&install_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start nixos-install.");

    let stdout_handle = install_cmd.stdout.take().expect("Failed to open stdout pipe.");
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout_handle);
        for line in reader.lines().flatten() {
            info!("{}", line);
        }
    });

    let stderr_handle = install_cmd.stderr.take().expect("Failed to open stderr pipe.");
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr_handle);
        for line in reader.lines().flatten() {
            info!("{}", line);
        }
    });

    // Wait for the threads capturing output to finish before returning
    stdout_thread.join().expect("Failed to join stdout thread.");
    stderr_thread.join().expect("Failed to join stderr thread.");

    // Wait for the installation process to complete
    let status = install_cmd.wait();
    let exit_code = match status {
        Ok(exit_status) => match exit_status.code() {
            Some(code) => {
                code
            }
            None => {
                info!("Process terminated without an exit code.");
                -1
            }
        },
        Err(err) => {
            error!("Failed to wait for process: {}", err);
            -1
        }
    };

    /* Nix Channel cannot be run in chroot (File not found despite exists)
       Users must run it once they login on the system.
    // Update nix channels on the target system after installation
    exec_eval(
        exec_nixroot(
            "nix-channel",
            vec![
                String::from("--update"),
            ],
        ),
        "Update nix channels on the target system",
    );
    */

    exit_code
}
