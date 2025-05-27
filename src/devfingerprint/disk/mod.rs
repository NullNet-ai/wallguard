#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "linux")]
mod linux;

#[allow(unreachable_code)]
pub fn disks_fingerprint() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        return linux::disks_fingerprint();
    }
    #[cfg(target_os = "freebsd")]
    {
        return freebsd::disks_fingerprint();
    }

    None
}
