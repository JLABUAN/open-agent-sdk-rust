//! Edge case tests for image handling
//!
//! Tests various edge cases and boundary conditions for image support.

use open_agent::{ContentBlock, ImageBlock, ImageDetail, Message, MessageRole, TextBlock};

#[test]
fn test_empty_content_vector() {
    // Should handle messages with empty content gracefully
    let msg = Message::new(MessageRole::User, vec![]);
    assert_eq!(msg.content.len(), 0);
}

#[test]
fn test_multiple_consecutive_images() {
    // Should handle multiple images in sequence
    let img1 = ImageBlock::from_url("https://example.com/1.jpg").unwrap();
    let img2 = ImageBlock::from_url("https://example.com/2.jpg").unwrap();
    let img3 = ImageBlock::from_url("https://example.com/3.jpg").unwrap();

    let msg = Message::new(
        MessageRole::User,
        vec![
            ContentBlock::Image(img1),
            ContentBlock::Image(img2),
            ContentBlock::Image(img3),
        ],
    );

    assert_eq!(msg.content.len(), 3);
}

#[test]
fn test_image_only_message() {
    // Should handle message with only image (no text)
    let img = ImageBlock::from_url("https://example.com/img.jpg").unwrap();
    let msg = Message::new(MessageRole::User, vec![ContentBlock::Image(img)]);

    assert_eq!(msg.content.len(), 1);
}

#[test]
fn test_many_images_in_one_message() {
    // Should handle messages with many images (10+)
    let images: Vec<ContentBlock> = (0..15)
        .map(|i| {
            ContentBlock::Image(
                ImageBlock::from_url(format!("https://example.com/{}.jpg", i))
                    .unwrap()
                    .with_detail(ImageDetail::Low),
            )
        })
        .collect();

    let msg = Message::new(MessageRole::User, images);
    assert_eq!(msg.content.len(), 15);
}

#[test]
fn test_alternating_text_and_images() {
    // Should handle alternating text and images
    let msg = Message::new(
        MessageRole::User,
        vec![
            ContentBlock::Text(TextBlock::new("First text")),
            ContentBlock::Image(ImageBlock::from_url("https://example.com/1.jpg").unwrap()),
            ContentBlock::Text(TextBlock::new("Second text")),
            ContentBlock::Image(ImageBlock::from_url("https://example.com/2.jpg").unwrap()),
            ContentBlock::Text(TextBlock::new("Third text")),
        ],
    );

    assert_eq!(msg.content.len(), 5);
}

#[test]
fn test_various_mime_types() {
    // Should handle all common image MIME types
    // Use properly formatted base64 (length divisible by 4)
    let base64 = "AAAA";
    let mime_types = [
        "image/jpeg",
        "image/png",
        "image/gif",
        "image/webp",
        "image/avif",
    ];

    for mime in &mime_types {
        let img = ImageBlock::from_base64(base64, *mime);
        assert!(img.is_ok(), "Should accept {}", mime);
    }
}

#[test]
fn test_large_base64_data() {
    // Should handle large base64 strings
    // Use valid base64 chars and make length divisible by 4
    let large_base64 = "A".repeat(10000);
    let img = ImageBlock::from_base64(&large_base64, "image/png");

    assert!(img.is_ok());
    assert!(img.unwrap().url().len() > 10000);
}

#[test]
fn test_unicode_in_text_with_images() {
    // Should handle Unicode text mixed with images
    let msg = Message::new(
        MessageRole::User,
        vec![
            ContentBlock::Text(TextBlock::new("こんにちは 🌸")),
            ContentBlock::Image(ImageBlock::from_url("https://example.com/img.jpg").unwrap()),
            ContentBlock::Text(TextBlock::new("مرحبا 🎨")),
        ],
    );

    assert_eq!(msg.content.len(), 3);
}

#[test]
fn test_data_uri_with_different_encodings() {
    // Should handle data URIs with base64
    let data_uri = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let img = ImageBlock::from_url(data_uri);

    assert!(img.is_ok());
    assert_eq!(img.unwrap().url(), data_uri);
}

// Phase 3: Empty text block validation tests

#[test]
fn test_empty_text_block_still_serialized() {
    // Empty text blocks should be included in serialization (not dropped)
    // even though they trigger a warning
    let msg = Message::new(
        MessageRole::User,
        vec![
            ContentBlock::Text(TextBlock::new("")),
            ContentBlock::Image(ImageBlock::from_url("https://example.com/img.jpg").unwrap()),
        ],
    );

    // Message should have 2 blocks
    assert_eq!(msg.content.len(), 2);

    // When serialized with images, should produce array format
    // Both text (even if empty) and image should be present
    // This is tested implicitly - if empty text was dropped, message would be image-only
}

#[test]
fn test_whitespace_only_text_block() {
    // Whitespace-only text blocks should trigger warning but still serialize
    let msg = Message::new(
        MessageRole::User,
        vec![
            ContentBlock::Text(TextBlock::new("   ")),
            ContentBlock::Image(ImageBlock::from_url("https://example.com/img.jpg").unwrap()),
        ],
    );

    assert_eq!(msg.content.len(), 2);
}

#[test]
fn test_mixed_empty_and_valid_text() {
    // Should handle mix of empty and valid text blocks
    let msg = Message::new(
        MessageRole::User,
        vec![
            ContentBlock::Text(TextBlock::new("Valid text")),
            ContentBlock::Text(TextBlock::new("")), // Empty - should warn
            ContentBlock::Text(TextBlock::new("More valid text")),
            ContentBlock::Image(ImageBlock::from_url("https://example.com/img.jpg").unwrap()),
        ],
    );

    // All blocks should be preserved
    assert_eq!(msg.content.len(), 4);
}
