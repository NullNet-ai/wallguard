use nullnet_libdatastore::{CreateBody, CreateParams, CreateRequest, Query};

#[derive(Debug, Default)]
pub struct CreateRequestBuilder {
    table: Option<String>,
    durability: Option<String>,
    pluck: Option<String>,
    record: Option<String>,
}

impl CreateRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn durability(mut self, durability: impl Into<String>) -> Self {
        self.durability = Some(durability.into());
        self
    }

    pub fn pluck<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let joined = fields
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>()
            .join(",");
        self.pluck = Some(joined);
        self
    }

    pub fn record(mut self, value: impl Into<String>) -> Self {
        self.record = Some(value.into());
        self
    }

    pub fn build(self) -> CreateRequest {
        CreateRequest {
            params: Some(CreateParams {
                table: self.table.unwrap_or_default(),
            }),
            query: Some(Query {
                pluck: self.pluck.unwrap_or_default(),
                durability: self.durability.unwrap_or_default(),
            }),
            body: Some(CreateBody {
                record: self.record.unwrap_or_default(),
            }),
        }
    }
}
