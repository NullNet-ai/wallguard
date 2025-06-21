use nullnet_libdatastore::{DeleteQuery, DeleteRequest, Params};

#[derive(Debug, Default)]
pub struct DeleteRequestBuilder {
    id: Option<String>,
    table: Option<String>,
    is_root: bool,
    is_permanent: bool,
}

impl DeleteRequestBuilder {
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

    pub fn performed_by_root(mut self, value: bool) -> Self {
        self.is_root = value;
        self
    }
    pub fn permanent(mut self, val: bool) -> Self {
        self.is_permanent = val;
        self
    }

    pub fn build(self) -> DeleteRequest {
        DeleteRequest {
            params: Some(Params {
                id: self.id.unwrap_or_default(),
                table: self.table.unwrap_or_default(),
                r#type: if self.is_root {
                    String::from("root")
                } else {
                    String::new()
                },
            }),
            query: Some(DeleteQuery {
                is_permanent: if self.is_permanent {
                    String::from("true")
                } else {
                    String::from("false")
                },
            }),
        }
    }
}
