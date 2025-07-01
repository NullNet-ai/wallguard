use crate::app_context::AppContext;
use indexmap::IndexSet;
use nullnet_liberror::Error;
use nullnet_libipinfo::{ApiFields, IpInfoHandler, IpInfoProvider};
use std::net::IpAddr;
use std::sync::mpsc::Receiver;
use tokio::runtime::Handle;

pub fn ip_info_handler(
    rx: &Receiver<Option<IpAddr>>,
    cache_size: usize,
    rt_handle: &Handle,
    context: AppContext,
) {
    let mut ip_cache = IpCache::new(cache_size);
    for ip in rx.iter().flatten() {
        let is_cached = ip_cache.contains(ip);
        ip_cache.refresh(ip);
        if !is_cached {
            let context = context.clone();
            rt_handle.spawn(async move {
                get_and_store_ip_info(ip, context).await.unwrap_or_default();
            });
        }
    }
}

async fn get_and_store_ip_info(ip: IpAddr, context: AppContext) -> Result<(), Error> {
    let ip = ip.to_string();

    let token = context.sysdev_token_provider.get().await?;

    let is_stored = context
        .datastore
        .is_ip_info_stored(ip.as_str(), &token.jwt)
        .await?;

    if !is_stored {
        log::info!("Looking up IP information for {ip}");
        let ip_info = HANDLER.lookup(&ip).await?;
        log::info!("Looked up IP information for {ip}: {ip_info:?}");
        context
            .datastore
            .create_ip_info(&token.jwt, &ip_info, &ip)
            .await?;
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

static HANDLER: std::sync::LazyLock<IpInfoHandler> = std::sync::LazyLock::new(|| {
    #[cfg(not(debug_assertions))]
    let url = "https://ipapi.co/{ip}/json/?key={api_key}";
    #[cfg(debug_assertions)]
    let url = "https://ipapi.co/{ip}/json";

    let api_key = std::env::var("IP_INFO_API_KEY").unwrap_or_else(|_| {
        log::warn!("IP_INFO_API_KEY environment variable not set");
        String::new()
    });

    IpInfoHandler::new(vec![IpInfoProvider::new_api_provider(
        url,
        &api_key,
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
