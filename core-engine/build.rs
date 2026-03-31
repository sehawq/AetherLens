use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../shared/proto/aether.proto")?;
    emit_windows_packet_sdk_links();
    Ok(())
}

fn emit_windows_packet_sdk_links() {
    // 👉 chỉ chạy trên Windows + feature bật
    if !cfg!(target_os = "windows") {
        return;
    }
    if std::env::var_os("CARGO_FEATURE_PACKET_CAPTURE").is_none() {
        return;
    }

    let mut candidates: HashSet<PathBuf> = HashSet::new();

    // 🔥 1. ENV override (ưu tiên cao nhất)
    if let Some(dir) = std::env::var_os("NPCAP_SDK_DIR") {
        candidates.insert(PathBuf::from(dir));
    }

    // 🔥 2. Detect theo workspace
    if let Ok(cwd) = std::env::current_dir() {
        let base = cwd.join("..").join("npcap-sdk").join("Lib");
        candidates.insert(base.clone());
        candidates.insert(base.join(arch_subdir()));
    }

    // 🔥 3. Default install paths
    let common_paths = [
        r"C:\Npcap-SDK\Lib",
        r"C:\Npcap\Lib",
        r"C:\WpdPack\Lib",
        r"C:\Program Files\Npcap-SDK\Lib",
        r"C:\Program Files (x86)\Npcap-SDK\Lib",
    ];

    for p in common_paths {
        candidates.insert(PathBuf::from(p));
        candidates.insert(PathBuf::from(p).join(arch_subdir()));
    }

    // 🔥 4. ProgramFiles dynamic
    if let Some(pf) = std::env::var_os("ProgramFiles") {
        candidates.insert(Path::new(&pf).join("Npcap-SDK").join("Lib"));
    }

    if let Some(pf86) = std::env::var_os("ProgramFiles(x86)") {
        candidates.insert(Path::new(&pf86).join("Npcap-SDK").join("Lib"));
    }

    // 🔥 5. Find valid SDK
    let selected = candidates.into_iter().find(|p| has_required_libs(p));

    match selected {
        Some(path) => {
            println!("cargo:rustc-link-search=native={}", path.display());
            println!("cargo:rustc-link-lib=dylib=Packet");
            println!("cargo:rustc-link-lib=dylib=wpcap");

            println!("cargo:warning=Npcap SDK found at {}", path.display());
        }
        None => {
            println!("cargo:warning=Npcap SDK not found!");
            println!("cargo:warning=Set NPCAP_SDK_DIR to folder containing Packet.lib + wpcap.lib");
        }
    }
}

fn has_required_libs(path: &Path) -> bool {
    path.join("Packet.lib").exists() && path.join("wpcap.lib").exists()
}

fn arch_subdir() -> &'static str {
    match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "x64",
        Ok("x86") => "x86",
        _ => "",
    }
}
