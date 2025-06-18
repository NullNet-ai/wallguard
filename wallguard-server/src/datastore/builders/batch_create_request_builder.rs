use nullnet_libdatastore::{BatchCreateBody, BatchCreateRequest, CreateParams, Query};

#[derive(Debug, Default)]
pub struct BatchCreateRequestBuilder {
    table: Option<String>,
    pluck: Option<String>,
    durability: Option<String>,
    records: Option<String>,
    entity_prefix: Option<String>,
}

impl BatchCreateRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn pluck(mut self, pluck: impl Into<String>) -> Self {
        self.pluck = Some(pluck.into());
        self
    }

    pub fn durability(mut self, durability: impl Into<String>) -> Self {
        self.durability = Some(durability.into());
        self
    }

    pub fn records(mut self, records: impl Into<String>) -> Self {
        self.records = Some(records.into());
        self
    }

    pub fn entity_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.entity_prefix = Some(prefix.into());
        self
    }

    pub fn build(self) -> BatchCreateRequest {
        BatchCreateRequest {
            params: Some(CreateParams {
                table: self.table.unwrap_or_default(),
            }),
            query: Some(Query {
                pluck: self.pluck.unwrap_or_default(),
                durability: self.durability.unwrap_or_else(|| "soft".into()),
            }),
            body: Some(BatchCreateBody {
                records: self.records.unwrap_or_default(),
            }),
        }
    }
}
