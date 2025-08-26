#![cfg(feature = "location")]

use anyhow::{anyhow, Context, Error};
use std::fmt::{self, Display};

#[test]
fn test_where_info_with_anyhow_macro() {
    let error = anyhow!("test error message");
    let info = error.where_info();
    
    assert!(info.is_some(), "where_info() should return Some when location feature is enabled");
    let info_str = info.unwrap();
    assert!(info_str.contains("test error message"), "Info should contain the error message");
    assert!(info_str.contains("at "), "Info should contain location prefix");
    assert!(info_str.contains(":"), "Info should contain line and column numbers");
}

#[test]
fn test_where_info_with_error_new() {
    let error = Error::new(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
    let info = error.where_info();
    
    assert!(info.is_some(), "where_info() should return Some when location feature is enabled");
    let info_str = info.unwrap();
    assert!(info_str.contains("file not found"), "Info should contain the error message");
    assert!(info_str.contains("at "), "Info should contain location prefix");
    assert!(info_str.contains(":"), "Info should contain line and column numbers");
}

#[test]
fn test_where_info_with_context() {
    let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let result: Result<(), std::io::Error> = Err(error);
    let error = result.context("failed to read config file").unwrap_err();
    let info = error.where_info();
    
    assert!(info.is_some(), "where_info() should return Some when location feature is enabled");
    let info_str = info.unwrap();
    assert!(info_str.contains("failed to read config file"), "Info should contain the context message");
    assert!(info_str.contains("at "), "Info should contain location prefix");
    assert!(info_str.contains(":"), "Info should contain line and column numbers");
}

#[test]
fn test_where_info_format() {
    let error = anyhow!("format test");
    let info = error.where_info().unwrap();
    
    // Check that the format matches expected pattern: "Error occurred: {message} (at {location})"
    assert!(info.starts_with("Error occurred: "), "Info should start with 'Error occurred: '");
    assert!(info.contains("format test"), "Info should contain the error message");
    
    // Find the location part
    let location_start = info.find("(at ").unwrap();
    let location_part = &info[location_start..];
    assert_eq!(location_part.ends_with(")"), true, "Location part should end with closing parenthesis");
    
    // Extract just the location (without "(at " and ")")
    let location_content = &location_part[4..location_part.len()-1];
    let parts: Vec<&str> = location_content.split(':').collect();
    assert_eq!(parts.len(), 3, "Location should have format file:line:column");
    
    // Check that line and column are numbers
    assert!(parts[1].parse::<u32>().is_ok(), "Line number should be a valid u32");
    assert!(parts[2].parse::<u32>().is_ok(), "Column number should be a valid u32");
}

#[test]
fn test_where_info_different_error_types() {
    // Test with anyhow! macro (captures location)
    let error = anyhow!("string error");
    let info = error.where_info();
    assert!(info.is_some());
    assert!(info.unwrap().contains("string error"));
    
    // Test with anyhow! macro for &str
    let error = anyhow!("str error");
    let info = error.where_info();
    assert!(info.is_some());
    assert!(info.unwrap().contains("str error"));
    
    // Test with custom error type using Error::new (captures location)
    #[derive(Debug)]
    struct CustomError;
    
    impl Display for CustomError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "custom error")
        }
    }
    
    impl std::error::Error for CustomError {}
    
    let error = Error::new(CustomError);
    let info = error.where_info();
    assert!(info.is_some());
    assert!(info.unwrap().contains("custom error"));
}

#[test]
fn test_where_info_consistency() {
    // Create multiple errors and ensure they have different locations
    let error1 = anyhow!("first error");
    let info1 = error1.where_info().unwrap();
    
    let error2 = anyhow!("second error");
    let info2 = error2.where_info().unwrap();
    
    // The locations should be different (different line numbers)
    assert_ne!(info1, info2, "Different errors should have different location info");
    
    // But both should contain their respective messages
    assert!(info1.contains("first error"));
    assert!(info2.contains("second error"));
}

#[test]
fn test_where_info_with_chained_errors() {
    // Note: In anyhow, all context calls lose location information
    // This is a limitation of Error::context method implementation, it always passes None as location parameter
    let original_error = anyhow!("connection failed");
    let result1: Result<(), Error> = Err(original_error);
    let error1 = result1.context("network operation failed").unwrap_err();
    let info1 = error1.where_info();
    
    // Check first level context call - loses location information due to anyhow's implementation limitation
    assert!(info1.is_none(), "context calls in anyhow lose location information");
    
    // Test second level context
    let result2: Result<(), Error> = Err(error1);
    let chained_error = result2.context("application error").unwrap_err();
    let info2 = chained_error.where_info();
    
    // Check second level context call - also loses location information
    assert!(info2.is_none(), "chained context in anyhow loses location information");
}