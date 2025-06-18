use std::collections::HashMap;

use nullnet_libdatastore::AdvanceFilter;
use nullnet_libdatastore::GetByFilterBody;
use nullnet_libdatastore::GetByFilterRequest;
use nullnet_libdatastore::Join;
use nullnet_libdatastore::MultipleSort;
use nullnet_libdatastore::Params;

#[derive(Debug, Default)]
pub struct GetByFilterRequestBuilder {
    id: Option<String>,
    table: Option<String>,
    pluck: Vec<String>,
    order_by: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
    order_direction: Option<String>,
    advance_filters: Vec<AdvanceFilter>,
    joins: Vec<Join>,
    multiple_sort: Vec<MultipleSort>,
    pluck_object: HashMap<String, String>,
    date_format: Option<String>,
    is_root: bool,
}

impl GetByFilterRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn pluck(mut self, field: impl Into<String>) -> Self {
        self.pluck.push(field.into());
        self
    }

    pub fn plucks(mut self, fields: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.pluck.extend(fields.into_iter().map(Into::into));
        self
    }

    pub fn order_by(mut self, field: impl Into<String>) -> Self {
        self.order_by = Some(field.into());
        self
    }

    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn order_direction(mut self, direction: impl Into<String>) -> Self {
        self.order_direction = Some(direction.into());
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

    pub fn join(mut self, join: Join) -> Self {
        self.joins.push(join);
        self
    }

    pub fn joins(mut self, joins: impl IntoIterator<Item = Join>) -> Self {
        self.joins.extend(joins);
        self
    }

    pub fn multiple_sort(mut self, sort: MultipleSort) -> Self {
        self.multiple_sort.push(sort);
        self
    }

    pub fn multiple_sorts(mut self, sorts: impl IntoIterator<Item = MultipleSort>) -> Self {
        self.multiple_sort.extend(sorts);
        self
    }

    pub fn pluck_object(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.pluck_object.insert(key.into(), value.into());
        self
    }

    pub fn pluck_objects(
        mut self,
        entries: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (k, v) in entries {
            self.pluck_object.insert(k.into(), v.into());
        }
        self
    }

    pub fn date_format(mut self, format: impl Into<String>) -> Self {
        self.date_format = Some(format.into());
        self
    }

    pub fn performed_by_root(mut self, value: bool) -> Self {
        self.is_root = value;
        self
    }

    pub fn build(self) -> GetByFilterRequest {
        GetByFilterRequest {
            body: Some(GetByFilterBody {
                pluck: self.pluck,
                advance_filters: self.advance_filters,
                order_by: self.order_by.unwrap_or_default(),
                limit: self.limit.unwrap_or_default(),
                offset: self.offset.unwrap_or_default(),
                order_direction: self.order_direction.unwrap_or_default(),
                joins: self.joins,
                multiple_sort: self.multiple_sort,
                pluck_object: self.pluck_object,
                date_format: self.date_format.unwrap_or_default(),
            }),
            params: Some(Params {
                id: self.id.unwrap_or_default(),
                table: self.table.unwrap_or_default(),
                r#type: if self.is_root {
                    String::from("root")
                } else {
                    String::new()
                },
            }),
        }
    }
}
