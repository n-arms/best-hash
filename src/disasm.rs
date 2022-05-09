use std::fs::write;
use std::process::Command;
use std::slice;

pub fn print_objdump(buffer: *const u8, length: usize) {
    let slice = unsafe { slice::from_raw_parts(buffer, length) };

    write("temp.bin", slice).expect("write to temp.bin failed");

    let child = Command::new("objdump")
        .arg("-D")
        .arg("-b")
        .arg("binary")
        .arg("-mi386:x86-64")
        .arg("-M")
        .arg("intel")
        .arg("temp.bin")
        .output()
        .expect("objdump command failed");

    if !child.status.success() {
        panic!(
            "objdump command failed with stderr {}",
            String::from_utf8_lossy(&child.stderr)
        );
    }

    print!("{}", String::from_utf8_lossy(&child.stdout));
}
