use api_types::projects::{Facet, FacetField, FacetGroup, FacetOperator, SearchIndex};

pub trait AsString {
    fn as_string(&self) -> String;
}

/// converts Vec<FacetGroup> to a string like "[facet_group.as_string() | facet_group in facet_groups]"
impl AsString for Vec<FacetGroup> {
    fn as_string(&self) -> String {
        self.iter().map(|fg| fg.as_string()).collect::<Vec<_>>().join(",").to_string()
    }
}

/// Converts the FacetGroup: Vec<Facet> to a string like "[facet.as_string() | facet in facets]"
impl AsString for FacetGroup {
    fn as_string(&self) -> String {
        self.facets.iter().map(|f| f.as_string()).collect::<Vec<_>>().join(",").to_string()
    }
}

/// Converts the Facet: field:operator:value to a string like "facet=project:mod" for example
impl AsString for Facet {
    fn as_string(&self) -> String {
        format!("{}{}{}", self.field.as_string().as_str(), self.operator.as_string().as_str(), self.value)
    }
}

impl AsString for FacetField {
    fn as_string(&self) -> String {
        match self {
            FacetField::ProjectType => "project_type",
            FacetField::Categories => "categories",
            FacetField::Version => "versions",
            FacetField::ClientSide => "client_side",
            FacetField::ServerSide => "server_side",
            FacetField::OpenSource => "open_source",

            FacetField::Downloads => "downloads",
            FacetField::License => "license",
            FacetField::Author => "author",
            FacetField::ProjectId => "project_id",
            FacetField::Title => "title",
            FacetField::Follows => "follows",
            FacetField::Color => "color",
            FacetField::CreatedTimestamp => "created_timestamp",
            FacetField::ModifiedTimestamp => "modified_timestamp",
            FacetField::DateCreated => "date_created",
            FacetField::DateModified => "date_modified",
        }
        .to_string()
    }
}

impl AsString for FacetOperator {
    fn as_string(&self) -> String {
        match self {
            FacetOperator::Eq => ":", // Modrinth uses ":" for equality but also "=" is supported.
            FacetOperator::NotEq => "!=",
            FacetOperator::Greater => ">",
            FacetOperator::GreaterEq => ">=",
            FacetOperator::Less => "<",
            FacetOperator::LessEq => "<=",
        }
        .to_string()
    }
}

impl AsString for SearchIndex {
    fn as_string(&self) -> String {
        match self {
            SearchIndex::Relevance => "relevance",
            SearchIndex::Downloads => "downloads",
            SearchIndex::Follows => "follows",
            SearchIndex::Newest => "newest",
            SearchIndex::Updated => "updated",
        }
        .to_string()
    }
}
