//! Input validators for embedded-contract methods.
//!
//! Each validator takes an optional value and returns the first applicable
//! failure message, or `None` when the value is valid.

use crate::embedded::constants::{
    PILLAR_NAME_MAX_LENGTH, PILLAR_NAME_REGEXP, PROJECT_DESCRIPTION_MAX_LENGTH,
    PROJECT_NAME_MAX_LENGTH, TOKEN_DOMAIN_REGEXP, TOKEN_NAME_MAX_LENGTH, TOKEN_NAME_REGEXP,
    TOKEN_SYMBOL_EXCEPTIONS, TOKEN_SYMBOL_MAX_LENGTH, TOKEN_SYMBOL_REGEXP,
};
use regex::Regex;
use std::sync::LazyLock;

static TOKEN_NAME: LazyLock<Regex> = LazyLock::new(|| compile(TOKEN_NAME_REGEXP));
static TOKEN_SYMBOL: LazyLock<Regex> = LazyLock::new(|| compile(TOKEN_SYMBOL_REGEXP));
static TOKEN_DOMAIN: LazyLock<Regex> = LazyLock::new(|| compile(TOKEN_DOMAIN_REGEXP));
static PILLAR_NAME: LazyLock<Regex> = LazyLock::new(|| compile(PILLAR_NAME_REGEXP));

/// Compiles an embedded-contract pattern. The patterns are fixed constants known
/// to be valid, so a parse failure is a programming error.
#[allow(clippy::expect_used)]
fn compile(pattern: &str) -> Regex {
    Regex::new(pattern).expect("embedded-contract regex patterns are valid")
}

/// The message returned when the input is absent.
const NULL_MESSAGE: &str = "Value is null";

/// Validates a token name. Returns the failure message, or `None` when valid.
pub fn token_name(value: Option<&str>) -> Option<String> {
    let Some(value) = value else {
        return Some(String::from(NULL_MESSAGE));
    };
    if value.is_empty() {
        return Some(String::from("Token name can't be empty"));
    }
    if !TOKEN_NAME.is_match(value) {
        return Some(String::from(
            "Token name must contain only alphanumeric characters",
        ));
    }
    if value.len() > TOKEN_NAME_MAX_LENGTH {
        return Some(format!(
            "Token name must have maximum {TOKEN_NAME_MAX_LENGTH} characters"
        ));
    }
    None
}

/// Validates a token symbol. Returns the failure message, or `None` when valid.
pub fn token_symbol(value: Option<&str>) -> Option<String> {
    let Some(value) = value else {
        return Some(String::from(NULL_MESSAGE));
    };
    if value.is_empty() {
        return Some(String::from("Token symbol can't be empty"));
    }
    if !TOKEN_SYMBOL.is_match(value) {
        return Some(format!(
            "Token symbol must match pattern: {TOKEN_SYMBOL_REGEXP}"
        ));
    }
    if value.len() > TOKEN_SYMBOL_MAX_LENGTH {
        return Some(format!(
            "Token symbol must have maximum {TOKEN_SYMBOL_MAX_LENGTH} characters"
        ));
    }
    if TOKEN_SYMBOL_EXCEPTIONS.contains(&value) {
        return Some(String::from(
            "Token symbol must not be one of the following: ZNN, QSR",
        ));
    }
    None
}

/// Validates a token domain. Returns the failure message, or `None` when valid.
pub fn token_domain(value: Option<&str>) -> Option<String> {
    let Some(value) = value else {
        return Some(String::from(NULL_MESSAGE));
    };
    if value.is_empty() {
        return Some(String::from("Token domain can't be empty"));
    }
    if !TOKEN_DOMAIN.is_match(value) {
        return Some(String::from("Domain is not valid"));
    }
    None
}

/// Validates a pillar name. Returns the failure message, or `None` when valid.
pub fn pillar_name(value: Option<&str>) -> Option<String> {
    let Some(value) = value else {
        return Some(String::from(NULL_MESSAGE));
    };
    if value.is_empty() {
        return Some(String::from("Pillar name can't be empty"));
    }
    if !PILLAR_NAME.is_match(value) {
        return Some(format!(
            "Pillar name must match pattern : {PILLAR_NAME_REGEXP}"
        ));
    }
    if value.len() > PILLAR_NAME_MAX_LENGTH {
        return Some(format!(
            "Pillar name must have maximum {PILLAR_NAME_MAX_LENGTH} characters"
        ));
    }
    None
}

/// Validates a project name. Returns the failure message, or `None` when valid.
pub fn project_name(value: Option<&str>) -> Option<String> {
    let Some(value) = value else {
        return Some(String::from(NULL_MESSAGE));
    };
    if value.is_empty() {
        return Some(String::from("Project name can't be empty"));
    }
    if value.len() > PROJECT_NAME_MAX_LENGTH {
        return Some(format!(
            "Project name must have maximum {PROJECT_NAME_MAX_LENGTH} characters"
        ));
    }
    None
}

/// Validates a project description. Returns the failure message, or `None` when valid.
pub fn project_description(value: Option<&str>) -> Option<String> {
    let Some(value) = value else {
        return Some(String::from(NULL_MESSAGE));
    };
    if value.is_empty() {
        return Some(String::from("Project description can't be empty"));
    }
    if value.len() > PROJECT_DESCRIPTION_MAX_LENGTH {
        return Some(format!(
            "Project description must have maximum {PROJECT_DESCRIPTION_MAX_LENGTH} characters"
        ));
    }
    None
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn token_name_branches() {
        assert_eq!(token_name(Some("MyToken")), None);
        assert_eq!(
            token_name(Some("")).as_deref(),
            Some("Token name can't be empty")
        );
        assert_eq!(
            token_name(Some("bad name!")).as_deref(),
            Some("Token name must contain only alphanumeric characters")
        );
        assert_eq!(
            token_name(Some(&"a".repeat(41))).as_deref(),
            Some("Token name must have maximum 40 characters")
        );
        assert_eq!(token_name(None).as_deref(), Some("Value is null"));
    }

    #[test]
    fn token_symbol_branches() {
        assert_eq!(token_symbol(Some("ABC")), None);
        assert_eq!(
            token_symbol(Some("ZNN")).as_deref(),
            Some("Token symbol must not be one of the following: ZNN, QSR")
        );
        assert_eq!(
            token_symbol(Some("abc")).as_deref(),
            Some("Token symbol must match pattern: ^[A-Z0-9]+$")
        );
        assert_eq!(
            token_symbol(Some("")).as_deref(),
            Some("Token symbol can't be empty")
        );
    }

    #[test]
    fn token_domain_branches() {
        assert_eq!(token_domain(Some("zenon.network")), None);
        assert_eq!(
            token_domain(Some("not a domain")).as_deref(),
            Some("Domain is not valid")
        );
        assert_eq!(
            token_domain(Some("")).as_deref(),
            Some("Token domain can't be empty")
        );
    }

    #[test]
    fn pillar_name_branches() {
        assert_eq!(pillar_name(Some("Pillar1")), None);
        assert_eq!(
            pillar_name(Some("")).as_deref(),
            Some("Pillar name can't be empty")
        );
        assert_eq!(
            pillar_name(Some("bad name!")).as_deref(),
            Some("Pillar name must match pattern : ^([a-zA-Z0-9]+[-._]?)*[a-zA-Z0-9]$")
        );
        assert_eq!(
            pillar_name(Some(&"a".repeat(41))).as_deref(),
            Some("Pillar name must have maximum 40 characters")
        );
        assert_eq!(pillar_name(None).as_deref(), Some("Value is null"));
    }

    #[test]
    fn project_name_branches() {
        assert_eq!(project_name(Some("Project")), None);
        assert_eq!(
            project_name(Some("")).as_deref(),
            Some("Project name can't be empty")
        );
        assert_eq!(
            project_name(Some(&"a".repeat(31))).as_deref(),
            Some("Project name must have maximum 30 characters")
        );
        assert_eq!(project_name(None).as_deref(), Some("Value is null"));
    }

    #[test]
    fn project_description_branches() {
        assert_eq!(project_description(Some("desc")), None);
        assert_eq!(
            project_description(Some("")).as_deref(),
            Some("Project description can't be empty")
        );
        assert_eq!(
            project_description(Some(&"a".repeat(241))).as_deref(),
            Some("Project description must have maximum 240 characters")
        );
        assert_eq!(project_description(None).as_deref(), Some("Value is null"));
    }
}
