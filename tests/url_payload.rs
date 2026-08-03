#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use uptime_api::error::AppError;
use uptime_api::http::url::{CreateUrl, UpdateUrl, UrlPayload};

fn create_url(domain: &str, name: &str, check_interval_seconds: Option<i32>) -> CreateUrl {
    CreateUrl {
        domain: domain.to_string(),
        name: name.to_string(),
        check_interval_seconds,
        expected_content: None,
    }
}

fn update_url(
    domain: Option<&str>,
    name: Option<&str>,
    check_interval_seconds: Option<i32>,
) -> UpdateUrl {
    UpdateUrl {
        domain: domain.map(str::to_string),
        name: name.map(str::to_string),
        check_interval_seconds,
        expected_content: None,
        is_active: None,
    }
}

fn assert_bad_request(result: Result<(), AppError>, expected_message: &str) {
    match result {
        Err(AppError::BadRequest(message)) => assert_eq!(message, expected_message),
        other => panic!("expected BadRequest({expected_message:?}), got {other:?}"),
    }
}

mod create_url_validation {
    use super::*;

    #[test]
    fn accepts_a_valid_payload() {
        let payload = create_url("example.com", "Example", Some(60));
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn rejects_empty_domain() {
        let payload = create_url("", "Example", None);
        assert_bad_request(payload.validate(), "domain must not be empty");
    }

    #[test]
    fn rejects_whitespace_only_domain() {
        let payload = create_url("   ", "Example", None);
        assert_bad_request(payload.validate(), "domain must not be empty");
    }

    #[test]
    fn rejects_domain_without_a_tld() {
        let payload = create_url("localhost", "Example", None);
        assert_bad_request(payload.validate(), "domain must be valid TLD");
    }

    #[test]
    fn rejects_domain_with_numeric_tld() {
        let payload = create_url("example.123", "Example", None);
        assert_bad_request(payload.validate(), "domain must be valid TLD");
    }

    #[test]
    fn accepts_domain_with_subdomain() {
        let payload = create_url("status.example.co.uk", "Example", None);
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn rejects_empty_name() {
        let payload = create_url("example.com", "", None);
        assert_bad_request(payload.validate(), "name must not be empty");
    }

    #[test]
    fn rejects_zero_check_interval() {
        let payload = create_url("example.com", "Example", Some(0));
        assert_bad_request(
            payload.validate(),
            "check_interval_seconds must be positive",
        );
    }

    #[test]
    fn rejects_negative_check_interval() {
        let payload = create_url("example.com", "Example", Some(-30));
        assert_bad_request(
            payload.validate(),
            "check_interval_seconds must be positive",
        );
    }

    #[test]
    fn accepts_missing_check_interval() {
        let payload = create_url("example.com", "Example", None);
        assert!(payload.validate().is_ok());
    }
}

mod update_url_validation {
    use super::*;

    #[test]
    fn accepts_an_entirely_empty_patch() {
        let payload = update_url(None, None, None);
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn skips_unset_fields() {
        // Only `name` is being changed here; domain/interval are untouched
        // and must not be validated as if they were part of the patch.
        let payload = update_url(None, Some("New Name"), None);
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn validates_domain_when_present() {
        let payload = update_url(Some("not a domain"), None, None);
        assert_bad_request(payload.validate(), "domain must be valid TLD");
    }

    #[test]
    fn validates_name_when_present() {
        let payload = update_url(None, Some("   "), None);
        assert_bad_request(payload.validate(), "name must not be empty");
    }

    #[test]
    fn validates_check_interval_when_present() {
        let payload = update_url(None, None, Some(-1));
        assert_bad_request(
            payload.validate(),
            "check_interval_seconds must be positive",
        );
    }

    #[test]
    fn accepts_a_fully_populated_patch() {
        let payload = update_url(Some("example.com"), Some("Renamed"), Some(120));
        assert!(payload.validate().is_ok());
    }
}
