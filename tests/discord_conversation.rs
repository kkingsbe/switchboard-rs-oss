//! Unit tests for conversation state management
//!
//! Tests the ConversationManager:
//! - TTL functionality including-based conversation expiration
//! - Message history trimming
//! - Concurrent conversation isolation

#[cfg(feature = "discord")]
use switchboard::discord::conversation::{ConversationConfig, ConversationManager};

/// Test that conversations expire after the TTL duration.
///
/// This test verifies TTL behavior by:
/// 1. Creating a manager with TTL = 0 (always expired immediately)
/// 2. Demonstrating that any call to get_or_create_conversation creates a NEW conversation
/// 3. Verifying the conversation state is reset on each call
#[cfg(feature = "discord")]
#[tokio::test]
async fn test_ttl_expiration() {
    // Create a config with TTL = 0 minutes (always expired)
    // This tests the TTL expiration behavior - with TTL=0, conversations
    // are always considered expired, so each call creates a new conversation
    let config = ConversationConfig::new(30, 0); // 0 minutes = instant expiry
    let mut manager = ConversationManager::new(config);

    // Add some messages to user123
    manager.add_user_message("user123", "Hello");
    manager.add_user_message("user123", "How are you?");

    // Now, with TTL = 0, calling get_or_create should create a NEW conversation
    // because the existing one is always expired. So we can't verify the messages
    // using get_or_create - we need a different approach.

    // Let's verify via has_conversation and check that getting it creates new
    assert!(
        manager.has_conversation("user123"),
        "Conversation should exist"
    );

    // The key TTL behavior: with TTL=0, every access creates a new conversation
    // This is the core test - verify that calling get_or_create with TTL=0
    // always returns an empty conversation (new)
    let conv = manager.get_or_create_conversation("user123");

    // Since TTL=0, the old conversation expired and a new one was created
    // The new conversation should be empty
    assert!(
        conv.is_empty(),
        "With TTL=0, getting conversation creates new (empty) one"
    );

    // Add more messages - these should be to a NEW conversation
    manager.add_user_message("user123", "New message 1");
    manager.add_user_message("user123", "New message 2");

    // Get conversation again - should still be new (TTL=0)
    let conv2 = manager.get_or_create_conversation("user123");
    assert!(
        conv2.is_empty(),
        "With TTL=0, should always get fresh conversation"
    );

    // Test with different user - same behavior
    manager.add_user_message("user456", "First message for user456");
    let conv3 = manager.get_or_create_conversation("user456");
    assert!(
        conv3.is_empty(),
        "User456 should also get fresh conversation with TTL=0"
    );
}

/// Test message trimming when max_history is exceeded.
///
/// This test:
/// 1. Creates a manager with max_history = 3
/// 2. Adds 5 messages to a conversation
/// 3. Verifies only the last 3 messages remain
/// 4. Verifies the oldest 2 messages were removed
#[cfg(feature = "discord")]
#[tokio::test]
async fn test_message_trimming() {
    // Create manager with max_history = 3
    let config = ConversationConfig::new(3, 120); // max_history=3, ttl=120 min
    let mut manager = ConversationManager::new(config);

    // Add 5 messages
    manager.add_user_message("user123", "Message 1");
    manager.add_user_message("user123", "Message 2");
    manager.add_user_message("user123", "Message 3");
    manager.add_user_message("user123", "Message 4");
    manager.add_user_message("user123", "Message 5");

    // Get the conversation and check message count
    let conv = manager.get_or_create_conversation("user123");

    // Should only have 3 messages (the last 3)
    assert_eq!(
        conv.len(),
        3,
        "Should have exactly 3 messages after trimming"
    );

    // Verify the messages are the last 3 (Message 3, 4, 5)
    let messages = conv.messages();
    assert_eq!(
        messages[0].content, "Message 3",
        "First remaining message should be Message 3"
    );
    assert_eq!(
        messages[1].content, "Message 4",
        "Second remaining message should be Message 4"
    );
    assert_eq!(
        messages[2].content, "Message 5",
        "Third remaining message should be Message 5"
    );

    // Verify old messages are gone
    assert!(
        !messages.iter().any(|m| m.content == "Message 1"),
        "Message 1 should be trimmed"
    );
    assert!(
        !messages.iter().any(|m| m.content == "Message 2"),
        "Message 2 should be trimmed"
    );
}

/// Test that concurrent conversations for different users don't interfere.
///
/// This test:
/// 1. Creates multiple conversations for different users
/// 2. Adds messages to each using multiple threads
/// 3. Verifies each conversation has the correct messages
/// 4. Verifies conversations don't interfere with each other
#[cfg(feature = "discord")]
#[tokio::test]
async fn test_concurrent_conversations() {
    // Create manager with high max_history to avoid trimming during test
    let config = ConversationConfig::new(100, 120);
    let mut manager = ConversationManager::new(config);

    // Create multiple users
    let users = vec!["user1", "user2", "user3", "user4", "user5"];

    // Add different messages to each user's conversation
    for user in &users {
        let msg = format!("Hello from {}", user);
        manager.add_user_message(user, &msg);
        let msg2 = format!("Second message from {}", user);
        manager.add_user_message(user, &msg2);
    }

    // Verify each conversation has the correct messages
    for user in &users {
        let conv = manager.get_or_create_conversation(user);
        assert_eq!(conv.len(), 2, "{} should have 2 messages", user);

        let messages = conv.messages();
        assert!(
            messages.iter().any(|m| m.content.contains(user)),
            "{} should have their own message",
            user
        );
    }

    // Now test with concurrent threads - each thread adds to a different user
    use std::sync::{Arc, Mutex};
    let manager: Arc<Mutex<ConversationManager>> = Arc::new(Mutex::new(manager));
    let mut handles = vec![];

    // Create 10 threads, each adding messages to a specific user
    // We use different users to avoid synchronization issues
    for i in 0..10 {
        let user_id = format!("concurrent_user_{}", i); // 10 unique users
        let manager_clone = Arc::clone(&manager);

        let handle = std::thread::spawn(move || {
            let mut mgr = manager_clone.lock().unwrap();
            // Add 2 messages to this user's conversation
            let msg = format!("Thread message {} - 1", i);
            mgr.add_user_message(&user_id, &msg);
            let msg2 = format!("Thread message {} - 2", i);
            mgr.add_user_message(&user_id, &msg2);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    // Verify the concurrent conversations - each should have 2 messages
    let mut mgr = manager.lock().unwrap();
    for i in 0..10 {
        let user_id = format!("concurrent_user_{}", i);
        let conv = mgr.get_or_create_conversation(&user_id);

        // Each user should have 2 messages
        assert_eq!(
            conv.len(),
            2,
            "{} should have 2 messages after concurrent operations",
            user_id
        );

        // Verify messages contain expected content
        let messages = conv.messages();
        assert!(
            messages
                .iter()
                .any(|m| m.content.contains("Thread message")),
            "{} should have thread messages",
            user_id
        );
    }

    // Also verify the original users still have their messages
    for user in &users {
        let conv = mgr.get_or_create_conversation(user);
        assert_eq!(conv.len(), 2, "{} should still have 2 messages", user);
    }
}

/// Test that conversations for different users are completely isolated.
#[cfg(feature = "discord")]
#[tokio::test]
async fn test_conversation_isolation() {
    let config = ConversationConfig::new(10, 60);
    let mut manager = ConversationManager::new(config);

    // Create separate conversations for different users
    manager.add_user_message("alice", "Alice's first message");
    manager.add_user_message("alice", "Alice's second message");

    manager.add_user_message("bob", "Bob's message");

    manager.add_user_message("charlie", "Charlie message 1");
    manager.add_user_message("charlie", "Charlie message 2");
    manager.add_user_message("charlie", "Charlie message 3");

    // Verify each conversation has correct messages
    // (Need to borrow manager mutably only once at a time)
    let alice_len = {
        let conv = manager.get_or_create_conversation("alice");
        assert_eq!(conv.len(), 2);
        assert!(conv.messages()[0].content.contains("Alice"));
        assert!(conv.messages()[1].content.contains("Alice"));
        conv.messages()
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
    };

    let bob_len = {
        let conv = manager.get_or_create_conversation("bob");
        assert_eq!(conv.len(), 1);
        assert!(conv.messages()[0].content.contains("Bob"));
        conv.messages()
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
    };

    let charlie_len = {
        let conv = manager.get_or_create_conversation("charlie");
        assert_eq!(conv.len(), 3);
        for msg in conv.messages() {
            assert!(msg.content.contains("Charlie"));
        }
        conv.messages()
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
    };

    // Verify they don't share any messages
    for alice_msg in &alice_len {
        assert!(!bob_len.contains(alice_msg));
        assert!(!charlie_len.contains(alice_msg));
    }

    for bob_msg in &bob_len {
        assert!(!alice_len.contains(bob_msg));
        assert!(!charlie_len.contains(bob_msg));
    }
}
