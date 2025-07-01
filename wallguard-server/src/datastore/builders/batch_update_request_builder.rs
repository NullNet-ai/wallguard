use nullnet_libdatastore::{AdvanceFilter, BatchUpdateBody, BatchUpdateRequest, Params};

#[derive(Debug, Default)]
pub struct BatchUpdateRequestBuilder {
    advance_filters: Vec<AdvanceFilter>,
    updates: Option<String>,
    id: Option<String>,
    table: Option<String>,
    is_root: bool,
}

impl BatchUpdateRequestBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn advance_filter(mut self, filter: AdvanceFilter) -> Self {
        self.advance_filters.push(filter);
        self
    }

    pub fn advance_filters(mut self, filters: impl IntoIterator<Item = AdvanceFilter>) -> Self {
        self.advance_filters.extend(filters);
        self
    }

    pub fn performed_by_root(mut self, value: bool) -> Self {
        self.is_root = value;
        self
    }

    pub fn updates(mut self, updates: impl Into<String>) -> Self {
        self.updates = Some(updates.into());
        self
    }

    pub fn build(self) -> BatchUpdateRequest {
        BatchUpdateRequest {
            params: Some(Params {
                id: self.id.unwrap_or_default(),
                table: self.table.unwrap_or_default(),
                r#type: if self.is_root {
                    String::from("root")
                } else {
                    String::new()
                },
            }),
            body: Some(BatchUpdateBody {
                advance_filters: self.advance_filters,
                updates: self.updates.unwrap_or_default(),
            }),
        }
    }
}
