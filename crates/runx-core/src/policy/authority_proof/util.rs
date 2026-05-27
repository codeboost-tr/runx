use runx_contracts::schema::NonEmptyString;

pub(super) fn non_empty_vec(values: Vec<String>) -> Vec<NonEmptyString> {
    values.into_iter().map(Into::into).collect()
}

pub(super) fn non_empty_option(value: Option<String>) -> Option<NonEmptyString> {
    value
        .filter(|value| !value.trim().is_empty())
        .map(Into::into)
}
