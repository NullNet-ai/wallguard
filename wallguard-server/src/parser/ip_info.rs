use crate::datastore::DatastoreWrapper;
use crate::utils::{ACCOUNT_ID, ACCOUNT_SECRET};
use indexmap::IndexSet;
use nullnet_liberror::Error;
use nullnet_libipinfo::{ApiFields, IpInfoHandler, IpInfoProvider};
use std::net::IpAddr;
use std::sync::mpsc::Receiver;
use tokio::runtime::Handle;

const IP_INFO_API_KEY: Option<&str> = option_env!("IP_INFO_API_KEY");

pub fn ip_info_handler(
    rx: Receiver<Option<IpAddr>>,
    cache_size: usize,
    rt_handle: &Handle,
    ds: &DatastoreWrapper,
) {
    let mut ip_cache = IpCache::new(cache_size);
    for ip in rx.iter().flatten() {
        let is_cached = ip_cache.contains(ip);
        ip_cache.refresh(ip);
        if !is_cached {
            let ds = ds.clone();
            rt_handle.spawn(async move {
                get_and_store_ip_info(ip, ds).await.unwrap_or_default();
            });
        }
    }
}

async fn get_and_store_ip_info(ip: IpAddr, mut ds: DatastoreWrapper) -> Result<(), Error> {
    let ip = ip.to_string();
    let token = ds
        .login(ACCOUNT_ID.to_string(), ACCOUNT_SECRET.to_string())
        .await?;

    let is_stored = ds.is_ip_info_stored(ip.as_str(), token.as_str()).await?;

    if !is_stored {
        log::info!("Looking up IP information for {ip}");
        let ip_info = HANDLER.lookup(&ip).await?;
        log::info!("Looked up IP information for {ip}: {ip_info:?}");
        ds.insert_ip_info(&ip, ip_info, &token).await?;
    }

    Ok(())
}

struct IpCache {
    cache: IndexSet<IpAddr>,
    size: usize,
}

impl IpCache {
    fn new(size: usize) -> Self {
        Self {
            cache: IndexSet::new(),
            size,
        }
    }

    fn refresh(&mut self, ip: IpAddr) {
        self.cache.shift_insert(0, ip);
        while self.cache.len() > self.size {
            self.cache.pop();
        }
    }

    fn contains(&self, ip: IpAddr) -> bool {
        self.cache.contains(&ip)
    }
}

static HANDLER: once_cell::sync::Lazy<IpInfoHandler> = once_cell::sync::Lazy::new(|| {
    #[cfg(not(debug_assertions))]
    let url = "https://ipapi.co/{ip}/json/?key={api_key}";
    #[cfg(debug_assertions)]
    let url = "https://ipapi.co/{ip}/json";

    let api_key = IP_INFO_API_KEY.unwrap_or({
        log::warn!("IP_INFO_API_KEY environment variable not set");
        ""
    });

    IpInfoHandler::new(vec![IpInfoProvider::new_api_provider(
        url,
        api_key,
        ApiFields {
            country: Some("/country"),
            asn: Some("/asn"),
            org: Some("/org"),
            continent_code: Some("/continent_code"),
            city: Some("/city"),
            region: Some("/region"),
            postal: Some("/postal"),
            timezone: Some("/timezone"),
        },
    )])
    .unwrap()
});
