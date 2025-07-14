use nullnet_libdatastore::AdvanceFilter;

#[derive(Debug, Default)]
pub struct AdvanceFilterBuilder {
    r#type: String,
    field: String,
    operator: String,
    entity: String,
    values: String,
}

impl AdvanceFilterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn r#type(mut self, value: impl Into<String>) -> Self {
        self.r#type = value.into();
        self
    }

    pub fn field(mut self, value: impl Into<String>) -> Self {
        self.field = value.into();
        self
    }

    pub fn operator(mut self, value: impl Into<String>) -> Self {
        self.operator = value.into();
        self
    }

    pub fn entity(mut self, value: impl Into<String>) -> Self {
        self.entity = value.into();
        self
    }

    pub fn values(mut self, value: impl Into<String>) -> Self {
        self.values = value.into();
        self
    }

    pub fn build(self) -> AdvanceFilter {
        AdvanceFilter {
            r#type: self.r#type,
            field: self.field,
            operator: self.operator,
            entity: self.entity,
            values: self.values,
        }
    }
}
