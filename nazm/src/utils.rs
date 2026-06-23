use rand::Rng;
use std::path::PathBuf;
use std::process::{Command, exit};

pub fn intro() {
    let intro = r#"

███▄▄▄▄      ▄████████  ▄███████▄    ▄▄▄▄███▄▄▄▄
███▀▀▀██▄   ███    ███ ██▀     ▄██ ▄██▀▀▀███▀▀▀██▄
███   ███   ███    ███       ▄███▀ ███   ███   ███
███   ███   ███    ███  ▀█▀▄███▀▄▄ ███   ███   ███
███   ███ ▀███████████   ▄███▀   ▀ ███   ███   ███
███   ███   ███    ███ ▄███▀       ███   ███   ███
███   ███   ███    ███ ███▄     ▄█ ███   ███   ███
 ▀█   █▀    ███    █▀   ▀████████▀  ▀█   ███   █▀

                                         𝖆𝖗𝖎𝖆2 𝖜𝖊𝖇 𝖚𝖎, 𝖇𝖚𝖙 𝖎𝖙'𝖘 𝖇𝖊𝖙𝖙𝖊𝖗!!
                                                "#;
    println!("{:}", intro);
}

/// Generates a valid 16-character hex string for Aria2 GIDs
pub fn generate_gid() -> String {
    let mut rng = rand::rng();
    loop {
        let bytes: [u8; 8] = rng.random();
        if bytes.iter().all(|&b| b == 0) {
            continue;
        }
        return bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
    }
}

pub fn default_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(".nazm"))
        .join("nazm")
}

pub fn default_aria2_dir() -> PathBuf {
    let base_dir = match dirs::download_dir() {
        Some(path) => path,
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };
    base_dir.join("aria2")
}

pub fn ensure_aria2_installation() {
    let result = Command::new("aria2c")
        .arg("--version")
        .output();

    match result {
        Ok(output) if output.status.success() => {
			tracing::info!("Aria2 is installed!!")
		}
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.trim().is_empty() {
                tracing::error!("Aria2 is installed but failed to run");
            } else {
                tracing::error!("Aria2 is installed but failed to run: {}", stderr.trim());
            }
            exit(1);
        }
        _ => {
            tracing::error!("Aria2 is not installed or not available in $PATH. Please install it before running NAZM!!");
            exit(1);
        }
    }
}
