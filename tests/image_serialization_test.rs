//! Integration tests for image serialization to OpenAI Vision API format
//!
//! Tests that OpenAIMessage and OpenAIContent properly serialize messages
//! containing images to the OpenAI Vision API format with array content structure.

use open_agent::{
    ContentBlock, ImageBlock, ImageDetail, Message, MessageRole, OpenAIContent, OpenAIContentPart,
    TextBlock,
};
use serde_json::Value;

#[test]
fn test_openai_content_text_serialization() {
    // GIVEN: OpenAIContent with text
    let content = OpenAIContent::Text("Hello world".to_string());

    // WHEN: Serialized to JSON
    let json = serde_json::to_string(&content).unwrap();

    // THEN: Should serialize as simple string
    assert_eq!(
        json, r#""Hello world""#,
        "Text-only content should serialize as simple string for backward compatibility"
    );
}

#[test]
fn test_openai_content_parts_serialization() {
    // GIVEN: OpenAIContent with parts array
    let parts = vec![
        OpenAIContentPart::text("Check this out:"),
        OpenAIContentPart::image_url("https://example.com/image.jpg", ImageDetail::High),
    ];
    let content = OpenAIContent::Parts(parts);

    // WHEN: Serialized to JSON
    let json = serde_json::to_value(&content).unwrap();

    // THEN: Should serialize as array with proper structure
    assert!(json.is_array(), "Parts should serialize as array");
    let array = json.as_array().unwrap();
    assert_eq!(array.len(), 2);

    // First part: text
    assert_eq!(array[0]["type"], "text");
    assert_eq!(array[0]["text"], "Check this out:");
    assert!(array[0].get("image_url").is_none());

    // Second part: image
    assert_eq!(array[1]["type"], "image_url");
    assert!(array[1].get("text").is_none());
    assert_eq!(
        array[1]["image_url"]["url"],
        "https://example.com/image.jpg"
    );
    assert_eq!(array[1]["image_url"]["detail"], "high");
}

#[test]
fn test_image_detail_serialization() {
    // GIVEN: Different ImageDetail levels
    let part_low = OpenAIContentPart::image_url("https://example.com/img.jpg", ImageDetail::Low);
    let part_high = OpenAIContentPart::image_url("https://example.com/img.jpg", ImageDetail::High);
    let part_auto = OpenAIContentPart::image_url("https://example.com/img.jpg", ImageDetail::Auto);

    // WHEN: Serialized
    let json_low = serde_json::to_value(&part_low).unwrap();
    let json_high = serde_json::to_value(&part_high).unwrap();
    let json_auto = serde_json::to_value(&part_auto).unwrap();

    // THEN: Detail levels should be lowercase strings
    assert_eq!(json_low["image_url"]["detail"], "low");
    assert_eq!(json_high["image_url"]["detail"], "high");
    assert_eq!(json_auto["image_url"]["detail"], "auto");
}

#[test]
fn test_base64_image_data_uri_format() {
    // GIVEN: Base64 image
    let base64_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let image = ImageBlock::from_base64(base64_data, "image/png").unwrap();

    // WHEN: URL is extracted
    let url = image.url();

    // THEN: Should use data URI format
    assert!(
        url.starts_with("data:image/png;base64,"),
        "Base64 images should use data URI format, got: {}",
        url
    );
    assert!(
        url.contains(base64_data),
        "Data URI should contain the base64 data"
    );
}

#[test]
fn test_text_only_message_to_openai_content() {
    // This test documents the expected behavior when implementing
    // message-to-OpenAIContent conversion logic

    // GIVEN: Message with only text
    let _message = Message::user("Simple text message");

    // WHEN: Converted to OpenAIContent (logic to be implemented)
    let expected_content = OpenAIContent::Text("Simple text message".to_string());

    // THEN: Should use Text variant (not Parts)
    let json = serde_json::to_string(&expected_content).unwrap();
    assert_eq!(json, r#""Simple text message""#);
}

#[test]
fn test_image_only_message_to_openai_content() {
    // This test documents the expected behavior when implementing
    // message-to-OpenAIContent conversion logic

    // GIVEN: Message with only an image
    let _message = Message::new(
        MessageRole::User,
        vec![ContentBlock::Image(
            ImageBlock::from_url("https://example.com/test.jpg")
                .unwrap()
                .with_detail(ImageDetail::High),
        )],
    );

    // WHEN: Converted to OpenAIContent (logic to be implemented)
    let expected_content = OpenAIContent::Parts(vec![OpenAIContentPart::image_url(
        "https://example.com/test.jpg",
        ImageDetail::High,
    )]);

    // THEN: Should use Parts variant with image_url
    let json: Value = serde_json::to_value(&expected_content).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0]["type"], "image_url");
    assert_eq!(json[0]["image_url"]["url"], "https://example.com/test.jpg");
    assert_eq!(json[0]["image_url"]["detail"], "high");
}

#[test]
fn test_mixed_content_message_to_openai_content() {
    // This test documents the expected behavior when implementing
    // message-to-OpenAIContent conversion logic

    // GIVEN: Message with text and image
    let _message = Message::new(
        MessageRole::User,
        vec![
            ContentBlock::Text(TextBlock::new("Look at this:")),
            ContentBlock::Image(
                ImageBlock::from_url("https://example.com/diagram.png")
                    .unwrap()
                    .with_detail(ImageDetail::Low),
            ),
            ContentBlock::Text(TextBlock::new("What do you see?")),
        ],
    );

    // WHEN: Converted to OpenAIContent (logic to be implemented)
    let expected_content = OpenAIContent::Parts(vec![
        OpenAIContentPart::text("Look at this:"),
        OpenAIContentPart::image_url("https://example.com/diagram.png", ImageDetail::Low),
        OpenAIContentPart::text("What do you see?"),
    ]);

    // THEN: Should preserve order and use Parts variant
    let json: Value = serde_json::to_value(&expected_content).unwrap();
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 3);

    assert_eq!(json[0]["type"], "text");
    assert_eq!(json[0]["text"], "Look at this:");

    assert_eq!(json[1]["type"], "image_url");
    assert_eq!(
        json[1]["image_url"]["url"],
        "https://example.com/diagram.png"
    );
    assert_eq!(json[1]["image_url"]["detail"], "low");

    assert_eq!(json[2]["type"], "text");
    assert_eq!(json[2]["text"], "What do you see?");
}

#[test]
fn test_multiple_images_in_content() {
    // GIVEN: Multiple images in Parts
    let expected_content = OpenAIContent::Parts(vec![
        OpenAIContentPart::text("Compare:"),
        OpenAIContentPart::image_url("https://example.com/img1.jpg", ImageDetail::Auto),
        OpenAIContentPart::image_url("https://example.com/img2.jpg", ImageDetail::Auto),
    ]);

    // WHEN: Serialized
    let json: Value = serde_json::to_value(&expected_content).unwrap();

    // THEN: All images should be included
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 3);
    assert_eq!(json[1]["type"], "image_url");
    assert_eq!(json[2]["type"], "image_url");
}
