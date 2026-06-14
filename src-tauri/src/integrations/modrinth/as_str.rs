use api_types::mods::{Facet, FacetField, FacetGroup, FacetOperator, ProjectSearch};

pub trait AsStr {
    fn as_str(&self) -> &str;
}

/// Converts the FacetGroup: Vec<Facet> to a string like "facet=project:mod&facet=version:1.0.0" for example
impl AsStr for FacetGroup {
    fn as_str(&self) -> String {
        self.facets
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Converts the Facet: field:operator:value to a string like "facet=project:mod" for example
impl AsStr for Facet {
    fn as_str(&self) -> String {
        format!(
            "{}{}{}",
            self.field.as_str(),
            self.operator.as_str(),
            self.value
        )
    }
}

impl AsStr for FacetField {
    fn as_str(&self) -> &str {
        match self {
            FacetField::ProjectType => "project_type",
            FacetField::Category => "categories",
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
    }
}

impl AsStr for FacetOperator {
    fn as_str(&self) -> &str {
        match self {
            FacetOperator::Eq => ":", // Modrinth uses ":" for equality but also "=" is supported.
            FacetOperator::NotEq => "!=",
            FacetOperator::Greater => ">",
            FacetOperator::GreaterEq => ">=",
            FacetOperator::Less => "<",
            FacetOperator::LessEq => "<=",
        }
    }
}
