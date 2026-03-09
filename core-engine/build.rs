use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../shared/proto/aether.proto")?;
    emit_windows_packet_sdk_links();
    Ok(())
}

fn emit_windows_packet_sdk_links() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }
    if std::env::var_os("CARGO_FEATURE_PACKET_CAPTURE").is_none() {
        return;
    }

    let mut candidates = Vec::new();

    if let Some(sdk_dir) = std::env::var_os("NPCAP_SDK_DIR") {
        let path = PathBuf::from(&sdk_dir);
        // ... (removed print debugs for cleanliness)
        candidates.push(path);
    }

    // Auto-detect from project root (relative to core-engine/)
    if let Ok(cwd) = std::env::current_dir() {
        // core-engine/../npcap-sdk/Lib/x64
        candidates.push(cwd.join("..").join("npcap-sdk").join("Lib").join("x64"));
        // core-engine/../npcap-sdk/Lib
        candidates.push(cwd.join("..").join("npcap-sdk").join("Lib"));
    }

    candidates.push(PathBuf::from(r"C:\Npcap-SDK\Lib"));
    candidates.push(PathBuf::from(r"C:\Npcap\Lib"));
    candidates.push(PathBuf::from(r"C:\WpdPack\Lib"));
    candidates.push(PathBuf::from(r"C:\WpdPack\Lib\x64"));
    candidates.push(PathBuf::from(r"C:\Program Files\Npcap-SDK\Lib"));
    candidates.push(PathBuf::from(r"C:\Program Files (x86)\Npcap-SDK\Lib"));

    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        candidates.push(Path::new(&program_files).join("Npcap-SDK").join("Lib"));
    }

    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        candidates.push(Path::new(&program_files_x86).join("Npcap-SDK").join("Lib"));
    }

    let selected = candidates
        .into_iter()
        .find(|path| path.join("Packet.lib").exists() && path.join("wpcap.lib").exists());

    if let Some(path) = selected {
        println!("cargo:rustc-link-search=native={}", path.display());
        println!("cargo:rustc-link-lib=dylib=Packet");
        println!("cargo:rustc-link-lib=dylib=wpcap");
        println!("cargo:warning=Npcap SDK found at {}", path.display());
    } else {
        println!("cargo:warning=Npcap SDK libs not found; set NPCAP_SDK_DIR to a Lib folder containing Packet.lib and wpcap.lib");
    }
}
