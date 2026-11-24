//! Regression test for the fork extension `AR_SUPPRESS_QUEUE_ACK`:
//! with the flag set, `re-review` still queues the forced review job
//! but posts no "queued" acknowledgement comment. Progress and the
//! outcome remain visible through the commit status and the review
//! itself, which the dispatcher and reviewer post separately.

extern crate alloc;

use alloc::sync::Arc;
use std::sync::Mutex;

use ar_chat::{ChatCommand, ChatContext, ChatHandler};
use ar_forgejo::Client as ForgejoClient;
use ar_index::InMemoryLearningsStore;
use ar_llm::Router;
use ar_orchestrator::{JobDispatcher, ReviewJob};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct RecordingDispatcher {
    seen: Arc<Mutex<Vec<ReviewJob>>>,
}

#[async_trait::async_trait]
impl JobDispatcher for RecordingDispatcher {
    async fn dispatch(&self, job: ReviewJob) {
        self.seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(job);
    }
}

fn ctx() -> ChatContext<'static> {
    ChatContext {
        owner: "alice",
        repo: "widgets",
        issue_number: 42,
        commenter_login: "alice",
        bot_login: "auto-review",
        reply_to_comment_id: None,
    }
}

#[tokio::test]
#[expect(
    clippy::default_numeric_fallback,
    reason = "serde_json::json! numbers here are untyped JSON wire values in the mocked Forgejo payload; type suffixes would edit the fixture without changing the bytes sent"
)]
#[expect(
    clippy::expect_used,
    reason = "a failed setup, request or decode step should fail this test at the named step; the asserted values stay as written"
)]
#[expect(
    clippy::unwrap_used,
    reason = "a poisoned lock should fail this test at once; the asserted values stay as written"
)]
async fn re_review_with_suppressed_ack_queues_job_without_comment() {
    std::env::set_var("AR_SUPPRESS_QUEUE_ACK", "true");

    let server = MockServer::start().await;
    let forgejo = ForgejoClient::new(&server.uri(), "tok").expect("client");
    let learnings = InMemoryLearningsStore::new();
    let router = Router::new();
    Mock::given(method("GET"))
        .and(path("/api/v1/repos/alice/widgets/pulls/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "number": 42,
            "title": "fix: thing",
            "body": "details",
            "draft": false,
            "head": {"ref": "topic", "sha": "deadbeef"},
            "base": {"ref": "main", "sha": "cafef00d"}
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/repos/alice/widgets/issues/42/comments"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;

    let seen = Arc::new(Mutex::new(Vec::new()));
    let dispatcher: Arc<dyn JobDispatcher> = Arc::new(RecordingDispatcher {
        seen: Arc::clone(&seen),
    });
    let handler = ChatHandler {
        host: &forgejo,
        llm: &router,
        learnings: &learnings,
        dispatcher: Some(dispatcher),
    };
    handler
        .handle(ctx(), ChatCommand::ReReview)
        .await
        .expect("ok");

    let queued = seen.lock().unwrap().clone();
    assert_eq!(queued.len(), 1, "the forced review job must still queue");
    assert!(queued[0].force, "ReReview must set force=true");

    let received = server.received_requests().await.expect("requests");
    let posts = received
        .iter()
        .filter(|request| request.method.as_str() == "POST")
        .count();
    assert_eq!(
        posts, 0,
        "AR_SUPPRESS_QUEUE_ACK must suppress the queued acknowledgement comment"
    );
}
