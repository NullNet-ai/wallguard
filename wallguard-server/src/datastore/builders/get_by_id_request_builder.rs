use nullnet_libdatastore::{GetByIdRequest, Params, Query};

pub struct GetByIdRequestBuilder {
    id: Option<String>,
    table: Option<String>,
    pluck: Option<String>,
    durability: Option<String>,
}

impl GetByIdRequestBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            table: None,
            pluck: None,
            durability: None,
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
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

    pub fn durability(mut self, durability: impl Into<String>) -> Self {
        self.durability = Some(durability.into());
        self
    }

    pub fn build(self) -> GetByIdRequest {
        GetByIdRequest {
            params: Some(Params {
                id: self.id.unwrap_or_default(),
                table: self.table.unwrap_or_default(),
            }),
            query: Some(Query {
                pluck: self.pluck.unwrap_or_default(),
                durability: self.durability.unwrap_or_default(),
            }),
        }
    }
}
