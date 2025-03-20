use std::net::IpAddr;
use std::sync::mpsc::Receiver;

pub fn ip_info_handler(rx: Receiver<(IpAddr, IpAddr)>, cache_size: usize) {
    for (src_ip, dst_ip) in rx {
        // todo: determine which IP address has to be looked up
        // ...
        // todo: ensure that the IP address is not already in the cache
        // ...
        // todo: check that the IP info is not already in the datastore and look it up if it isn't
        // ...
    }
}
