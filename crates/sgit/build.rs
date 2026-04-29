fn main() {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        use std::env;
        use std::path::Path;

        let out_dir = env::var("OUT_DIR").unwrap();
        let shim_path = "glibc_shim.c";
        
        if Path::new(shim_path).exists() {
            println!("cargo:rerun-if-changed={}", shim_path);
            
            // Compile the shim
            let status = Command::new("gcc")
                .args(&["-c", shim_path, "-o"])
                .arg(format!("{}/glibc_shim.o", out_dir))
                .status()
                .expect("Failed to run gcc");
            
            if status.success() {
                // Create static library
                Command::new("ar")
                    .args(&["rcs", "libglibc_shim.a", "glibc_shim.o"])
                    .current_dir(&out_dir)
                    .status()
                    .expect("Failed to run ar");
                
                println!("cargo:rustc-link-search=native={}", out_dir);
                println!("cargo:rustc-link-lib=static=glibc_shim");
            }
        }
    }
}
