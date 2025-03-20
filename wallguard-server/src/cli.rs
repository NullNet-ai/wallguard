use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// IP information local cache size
    #[arg(long, default_value_t = 1000)]
    pub ip_info_cache_size: usize,
}
