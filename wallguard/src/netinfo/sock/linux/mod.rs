use crate::netinfo::sock::SocketInfo;

mod inode_pid;
mod inode_sock;

pub(super) async fn get_sockets_info() -> Vec<SocketInfo> {
    let sock_map = inode_sock::build_inode_sock_map().await;
    let pid_map = inode_pid::build_inode_pid_map().await;

    let mut results = vec![];

    for (inode, (sockaddr, protocol)) in sock_map {
        if let Some(proc_name) = pid_map.get(&inode) {
            results.push(SocketInfo {
                process_name: proc_name.into(),
                sockaddr,
                protocol,
            });
        }
    }

    results
}
