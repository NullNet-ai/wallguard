use nullnet_libdatastore::{Params, Query, UpdateRequest};

pub struct UpdateRequestBuilder {
    id: Option<String>,
    table: Option<String>,
    pluck: Option<String>,
    durability: Option<String>,
    body: Option<String>,
}

impl UpdateRequestBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            table: None,
            pluck: None,
            durability: None,
            body: None,
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

    pub fn query(mut self, pluck: impl Into<String>, durability: impl Into<String>) -> Self {
        self.pluck = Some(pluck.into());
        self.durability = Some(durability.into());
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn build(self) -> UpdateRequest {
        UpdateRequest {
            params: Some(Params {
                id: self.id.expect("Missing 'id'"),
                table: self.table.expect("Missing 'table'"),
            }),
            query: Some(Query {
                pluck: self.pluck.unwrap_or_default(),
                durability: self.durability.unwrap_or_default(),
            }),
            body: self.body.unwrap_or_default(),
        }
    }
}
