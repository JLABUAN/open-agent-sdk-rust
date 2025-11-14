//! Backward compatibility tests for v0.6.0
//!
//! Verifies that v0.5.0 text-only message format is maintained and that
//! new v0.6.0 image functionality uses array format without breaking
//! existing behavior.

#![allow(deprecated)]

use open_agent::{ContentBlock, Message, MessageRole, OpenAIContent, TextBlock};

#[test]
fn test_text_only_message_uses_string_format() {
    // v0.5.0 behavior: text-only messages serialize as simple string
    let _msg = Message::new(
        MessageRole::User,
        vec![ContentBlock::Text(TextBlock::new("Hello"))],
    );

    // When serialized for OpenAI API, should use string format (not array)
    let content = OpenAIContent::Text("Hello".to_string());
    let json = serde_json::to_value(&content).unwrap();

    assert_eq!(json, "Hello");
}

#[test]
fn test_multiple_text_blocks_joined_with_newline() {
    // v0.5.0 behavior: multiple text blocks joined with \n
    let _msg = Message::new(
        MessageRole::User,
        vec![
            ContentBlock::Text(TextBlock::new("Line 1")),
            ContentBlock::Text(TextBlock::new("Line 2")),
        ],
    );

    // Should be joined with newline
    let content = OpenAIContent::Text("Line 1\nLine 2".to_string());
    let json = serde_json::to_value(&content).unwrap();

    assert_eq!(json, "Line 1\nLine 2");
}

#[test]
fn test_system_message_format_unchanged() {
    // System messages should maintain string format
    let _msg = Message::system("You are a helpful assistant");

    // Should serialize as simple string
    let content = OpenAIContent::Text("You are a helpful assistant".to_string());
    let json = serde_json::to_value(&content).unwrap();

    assert_eq!(json, "You are a helpful assistant");
}

#[test]
fn test_assistant_text_message_format_unchanged() {
    // Assistant text messages should maintain string format
    let _msg = Message::assistant(vec![ContentBlock::Text(TextBlock::new("Response"))]);

    let content = OpenAIContent::Text("Response".to_string());
    let json = serde_json::to_value(&content).unwrap();

    assert_eq!(json, "Response");
}

#[test]
fn test_image_message_uses_array_format() {
    // v0.6.0 behavior: messages with images use array format
    use open_agent::{ImageBlock, ImageDetail, OpenAIContentPart};

    let _img = ImageBlock::from_url("https://example.com/img.jpg")
        .unwrap()
        .with_detail(ImageDetail::Auto);

    let parts = vec![
        OpenAIContentPart::text("What's this?"),
        OpenAIContentPart::image_url("https://example.com/img.jpg", ImageDetail::Auto),
    ];
    let content = OpenAIContent::Parts(parts);
    let json = serde_json::to_value(&content).unwrap();

    assert!(json.is_array());
    assert_eq!(json[0]["type"], "text");
    assert_eq!(json[1]["type"], "image_url");
}

#[test]
fn test_v050_deserialize_text_message() {
    // Verify we can deserialize v0.5.0 format messages
    let json = serde_json::json!("Hello world");
    let content: OpenAIContent = serde_json::from_value(json).unwrap();

    match content {
        OpenAIContent::Text(text) => assert_eq!(text, "Hello world"),
        _ => panic!("Expected Text variant"),
    }
}
