use runx_contracts::JsonObject;

use crate::ValidationError;

use super::{
    CatalogAudience, CatalogKind, CatalogMetadata, CatalogVisibility, optional_string,
    required_string, validation_error,
};

pub(crate) fn validate_catalog_metadata(
    value: Option<JsonObject>,
    label: &str,
) -> Result<Option<CatalogMetadata>, ValidationError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let kind = match required_string(value.get("kind"), &format!("{label}.kind"))?.as_str() {
        "skill" => CatalogKind::Skill,
        "graph" => CatalogKind::Graph,
        _ => {
            return Err(validation_error(format!(
                "{label}.kind must be skill or graph."
            )));
        }
    };
    let audience =
        match required_string(value.get("audience"), &format!("{label}.audience"))?.as_str() {
            "public" => CatalogAudience::Public,
            "builder" => CatalogAudience::Builder,
            "operator" => CatalogAudience::Operator,
            _ => {
                return Err(validation_error(format!(
                    "{label}.audience must be public, builder, or operator."
                )));
            }
        };
    let visibility = match optional_string(value.get("visibility"), &format!("{label}.visibility"))?
        .as_deref()
    {
        Some("public") | None => CatalogVisibility::Public,
        Some("private") => CatalogVisibility::Private,
        Some(_) => {
            return Err(validation_error(format!(
                "{label}.visibility must be public or private."
            )));
        }
    };
    Ok(Some(CatalogMetadata {
        kind,
        audience,
        visibility,
    }))
}
