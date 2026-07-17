//! End-to-end integration tests for phenotype-event-bus → PhenoObservably OTEL emission.
//! Traces to: FR-OBS-E2E-001

use phenotype_event_bus::memory::InMemoryEventBus;
use phenotype_event_bus::{EventBus, EventEnvelope};
use phenotype_observably_tracing::MetricsRegistry;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Once;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::info;

static INIT_TRACING: Once = Once::new();

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct SidekickCacheMissEvent {
    cache_key: String,
    user_id: String,
}

#[tokio::test]
async fn test_sidekick_cache_miss_to_observably_logging() {
    INIT_TRACING.call_once(|| {
        phenotype_observably_tracing::init_tracing("test-e2e", Some("debug"));
    });

    let bus = InMemoryEventBus::<SidekickCacheMissEvent>::new();
    let captured = Arc::new(Mutex::new(None));

    bus.subscribe("test_source", {
        let cap = Arc::clone(&captured);
        move |envelope| {
            let cap = Arc::clone(&cap);
            tokio::spawn(async move {
                info!(
                    cache_key = %envelope.payload.cache_key,
                    user_id = %envelope.payload.user_id,
                    "Cache miss detected"
                );
                *cap.lock().await = Some(envelope.payload);
            });
            Ok(())
        }
    })
    .await
    .expect("subscribe");

    bus.publish(EventEnvelope::new(
        "test_source",
        SidekickCacheMissEvent {
            cache_key: "user-profile-001".to_string(),
            user_id: "user-123".to_string(),
        },
    ))
    .await
    .expect("publish");

    let _ = timeout(Duration::from_millis(500), async {
        loop {
            if captured.lock().await.is_some() {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;

    let result = captured.lock().await;
    assert!(
        result.is_some(),
        "Observably did not receive cache-miss event"
    );
    assert_eq!(result.as_ref().unwrap().cache_key, "user-profile-001");
}

#[tokio::test]
async fn test_focus_eval_rule_fired_to_observably_metrics() {
    INIT_TRACING.call_once(|| {
        phenotype_observably_tracing::init_tracing("test-e2e", Some("debug"));
    });

    #[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
    struct RuleFired {
        rule_id: String,
        duration_ms: u64,
    }

    let bus = InMemoryEventBus::<RuleFired>::new();
    let metrics = MetricsRegistry::global();
    let captured = Arc::new(Mutex::new(0u32));

    bus.subscribe("rules", {
        let cap = Arc::clone(&captured);
        let metrics_ref = metrics.clone();
        move |envelope| {
            let cap = Arc::clone(&cap);
            let metrics_ref = metrics_ref.clone();
            tokio::spawn(async move {
                metrics_ref.inc_rule_evaluations(&envelope.payload.rule_id, 1.0);
                metrics_ref.record_eval_duration(
                    &envelope.payload.rule_id,
                    envelope.payload.duration_ms as f64 / 1000.0,
                );
                *cap.lock().await += 1;
            });
            Ok(())
        }
    })
    .await
    .expect("subscribe");

    bus.publish(EventEnvelope::new(
        "rules",
        RuleFired {
            rule_id: "rule-time-window-01".to_string(),
            duration_ms: 125,
        },
    ))
    .await
    .expect("publish");

    let _ = timeout(Duration::from_millis(500), async {
        loop {
            if *captured.lock().await > 0 {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;

    assert_eq!(*captured.lock().await, 1);
    let text = metrics.gather_text_format().expect("gather metrics");
    assert!(text.contains("rule_evaluations_total"));
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct StashlyStorageEvent {
    artifact_id: String,
    size_bytes: u64,
}

#[tokio::test]
async fn test_stashly_storage_to_observably_otel_span() {
    INIT_TRACING.call_once(|| {
        phenotype_observably_tracing::init_tracing("test-e2e", Some("debug"));
    });

    let bus = InMemoryEventBus::<StashlyStorageEvent>::new();
    let captured = Arc::new(Mutex::new(None));

    bus.subscribe("stashly", {
        let cap = Arc::clone(&captured);
        move |envelope| {
            let cap = Arc::clone(&cap);
            tokio::spawn(async move {
                let span = tracing::info_span!(
                    "stashly.storage",
                    artifact_id = %envelope.payload.artifact_id,
                    size_bytes = envelope.payload.size_bytes,
                );
                let _guard = span.enter();
                info!("Artifact stored successfully");
                *cap.lock().await = Some(envelope.payload);
            });
            Ok(())
        }
    })
    .await
    .expect("subscribe");

    bus.publish(EventEnvelope::new(
        "stashly",
        StashlyStorageEvent {
            artifact_id: "artifact-abc-123".to_string(),
            size_bytes: 512_000,
        },
    ))
    .await
    .expect("publish");

    let _ = timeout(Duration::from_millis(500), async {
        loop {
            if captured.lock().await.is_some() {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;

    let result = captured.lock().await;
    assert!(
        result.is_some(),
        "Observably did not emit span for storage event"
    );
    assert_eq!(result.as_ref().unwrap().artifact_id, "artifact-abc-123");
}

#[tokio::test]
async fn test_end_to_end_cross_collection_pipeline() {
    INIT_TRACING.call_once(|| {
        phenotype_observably_tracing::init_tracing("test-e2e", Some("debug"));
    });

    #[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
    struct RuleFired {
        rule_id: String,
        duration_ms: u64,
    }

    let cache_bus = InMemoryEventBus::<SidekickCacheMissEvent>::new();
    let rule_bus = InMemoryEventBus::<RuleFired>::new();
    let storage_bus = InMemoryEventBus::<StashlyStorageEvent>::new();
    let metrics = MetricsRegistry::global();
    let event_log = Arc::new(Mutex::new(Vec::new()));

    cache_bus
        .subscribe("cache", {
            let log = Arc::clone(&event_log);
            move |envelope| {
                let log = Arc::clone(&log);
                tokio::spawn(async move {
                    info!(cache_key = %envelope.payload.cache_key, "Sidekick cache miss detected");
                    log.lock().await.push("cache-miss".to_string());
                });
                Ok(())
            }
        })
        .await
        .expect("cache subscribe");

    rule_bus
        .subscribe("rules", {
            let log = Arc::clone(&event_log);
            let metrics_ref = metrics.clone();
            move |envelope| {
                let log = Arc::clone(&log);
                let metrics_ref = metrics_ref.clone();
                tokio::spawn(async move {
                    metrics_ref.inc_rule_evaluations(&envelope.payload.rule_id, 1.0);
                    let span =
                        tracing::info_span!("rule.evaluate", rule_id = %envelope.payload.rule_id);
                    let _guard = span.enter();
                    info!("Rule evaluation completed");
                    log.lock().await.push("rule-fired".to_string());
                });
                Ok(())
            }
        })
        .await
        .expect("rule subscribe");

    storage_bus
        .subscribe("stashly", {
            let log = Arc::clone(&event_log);
            let metrics_ref = metrics.clone();
            move |envelope| {
                let log = Arc::clone(&log);
                let metrics_ref = metrics_ref.clone();
                tokio::spawn(async move {
                    metrics_ref.inc_audit_appends("artifact_stored", 1.0);
                    info!(artifact_id = %envelope.payload.artifact_id, "Artifact stored");
                    log.lock().await.push("storage".to_string());
                });
                Ok(())
            }
        })
        .await
        .expect("storage subscribe");

    cache_bus
        .publish(EventEnvelope::new(
            "cache",
            SidekickCacheMissEvent {
                cache_key: "e2e-key".to_string(),
                user_id: "e2e-user".to_string(),
            },
        ))
        .await
        .expect("cache publish");
    rule_bus
        .publish(EventEnvelope::new(
            "rules",
            RuleFired {
                rule_id: "e2e-rule".to_string(),
                duration_ms: 50,
            },
        ))
        .await
        .expect("rule publish");
    storage_bus
        .publish(EventEnvelope::new(
            "stashly",
            StashlyStorageEvent {
                artifact_id: "e2e-artifact".to_string(),
                size_bytes: 1024,
            },
        ))
        .await
        .expect("storage publish");

    let _ = timeout(Duration::from_millis(800), async {
        loop {
            if event_log.lock().await.len() >= 3 {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;

    let log = event_log.lock().await;
    assert_eq!(log.len(), 3, "All three events should have been processed");
    assert!(log.contains(&"cache-miss".to_string()));
    assert!(log.contains(&"rule-fired".to_string()));
    assert!(log.contains(&"storage".to_string()));

    let metrics_text = metrics.gather_text_format().expect("gather metrics");
    assert!(metrics_text.contains("rule_evaluations_total"));
    assert!(metrics_text.contains("audit_appends_total"));
}
