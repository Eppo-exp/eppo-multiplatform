use std::fs::File;

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

#[cfg(feature = "ahash")]
use eppo_core::ahash::{HashMap, HashMapExt};
#[cfg(not(feature = "ahash"))]
use std::collections::HashMap;

use eppo_core::ufc::UniversalFlagConfig;
use eppo_core::{eval::get_bandit_action, Configuration, ContextAttributes, SdkMetadata, Str};

fn criterion_benchmark(c: &mut Criterion) {
    // Load bandit flags configuration
    let flags = UniversalFlagConfig::from_json(
        SdkMetadata {
            name: "test",
            version: "0.1.0",
        },
        std::fs::read("../sdk-test-data/ufc/bandit-flags-v1.json").unwrap(),
    )
    .unwrap();

    // Load bandit models
    let bandits =
        serde_json::from_reader(File::open("../sdk-test-data/ufc/bandit-models-v1.json").unwrap())
            .unwrap();
    let configuration = Configuration::from_server_response(flags, Some(bandits));
    let now = Utc::now();

    let sdk_meta = SdkMetadata {
        name: "test",
        version: "0.1.0",
    };

    let mut group = c.benchmark_group("bandit");
    group.throughput(Throughput::Elements(1));

    // Small number of actions and attributes (2 actions)
    {
        // Subject attributes
        let subject_attributes = {
            let mut attrs = HashMap::new();
            attrs.insert("account_age".into(), 30.0.into());
            attrs.insert("gender_identity".into(), "female".into());
            ContextAttributes::from(attrs)
        };

        // Actions with their attributes (2 actions: nike and adidas)
        let actions: HashMap<Str, ContextAttributes> = {
            let mut actions_map = HashMap::new();

            let nike_attrs = {
                let mut attrs = HashMap::new();
                attrs.insert("brand_affinity".into(), 0.8.into());
                attrs.insert("loyalty_tier".into(), "gold".into());
                attrs.insert("zip".into(), "22203".into());
                ContextAttributes::from(attrs)
            };
            actions_map.insert("nike".into(), nike_attrs);

            let adidas_attrs = {
                let mut attrs = HashMap::new();
                attrs.insert("brand_affinity".into(), 0.6.into());
                attrs.insert("purchased_last_30_days".into(), "true".into());
                ContextAttributes::from(attrs)
            };
            actions_map.insert("adidas".into(), adidas_attrs);

            actions_map
        };

        group.bench_function("small-2-actions", |b| {
            b.iter_with_large_drop(|| {
                get_bandit_action(
                    black_box(Some(&configuration)),
                    black_box("banner_bandit_flag"),
                    black_box(&"subject1".into()),
                    black_box(&subject_attributes),
                    black_box(&actions),
                    black_box(&"control".into()),
                    black_box(now),
                    black_box(&sdk_meta),
                )
            })
        });
    }

    // Large number of actions and attributes (stress test: 100 actions)
    {
        // Generate 50 subject attributes
        let subject_attributes: ContextAttributes = {
            let mut attrs = HashMap::new();
            for i in 0..25 {
                attrs.insert(format!("numeric_attr_{}", i).into(), (i as f64).into());
            }
            for i in 0..25 {
                attrs.insert(
                    format!("categorical_attr_{}", i).into(),
                    format!("value_{}", i % 5).into(),
                );
            }
            ContextAttributes::from(attrs)
        };

        // Generate 100 actions, each with 20 attributes
        let actions: HashMap<Str, ContextAttributes> = {
            let mut actions_map = HashMap::new();
            for action_idx in 0..100 {
                let mut action_attrs = HashMap::new();
                for i in 0..10 {
                    action_attrs.insert(
                        format!("action_numeric_{}", i).into(),
                        ((action_idx * 10 + i) as f64 * 0.1).into(),
                    );
                }
                for i in 0..10 {
                    action_attrs.insert(
                        format!("action_categorical_{}", i).into(),
                        format!("action_value_{}", (action_idx + i) % 10).into(),
                    );
                }
                actions_map.insert(
                    format!("action_{}", action_idx).into(),
                    ContextAttributes::from(action_attrs),
                );
            }
            actions_map
        };

        group.bench_function("large-100-actions", |b| {
            b.iter_with_large_drop(|| {
                get_bandit_action(
                    black_box(Some(&configuration)),
                    black_box("banner_bandit_flag"),
                    black_box(&"subject1".into()),
                    black_box(&subject_attributes),
                    black_box(&actions),
                    black_box(&"control".into()),
                    black_box(now),
                    black_box(&sdk_meta),
                )
            })
        });
    }

    // Extra large test: 1000 actions (extreme stress test)
    {
        // Generate 100 subject attributes
        let subject_attributes: ContextAttributes = {
            let mut attrs = HashMap::new();
            for i in 0..50 {
                attrs.insert(format!("numeric_attr_{}", i).into(), (i as f64).into());
            }
            for i in 0..50 {
                attrs.insert(
                    format!("categorical_attr_{}", i).into(),
                    format!("value_{}", i % 10).into(),
                );
            }
            ContextAttributes::from(attrs)
        };

        // Generate 1000 actions, each with 100 attributes
        let actions: HashMap<Str, ContextAttributes> = {
            let mut actions_map = HashMap::new();
            for action_idx in 0..1000 {
                let mut action_attrs = HashMap::new();
                for i in 0..50 {
                    action_attrs.insert(
                        format!("action_numeric_{}", i).into(),
                        ((action_idx * 50 + i) as f64 * 0.1).into(),
                    );
                }
                for i in 0..50 {
                    action_attrs.insert(
                        format!("action_categorical_{}", i).into(),
                        format!("action_value_{}", (action_idx + i) % 20).into(),
                    );
                }
                actions_map.insert(
                    format!("action_{}", action_idx).into(),
                    ContextAttributes::from(action_attrs),
                );
            }
            actions_map
        };

        group.bench_function("xlarge-1000-actions", |b| {
            b.iter_with_large_drop(|| {
                get_bandit_action(
                    black_box(Some(&configuration)),
                    black_box("banner_bandit_flag"),
                    black_box(&"subject1".into()),
                    black_box(&subject_attributes),
                    black_box(&actions),
                    black_box(&"control".into()),
                    black_box(now),
                    black_box(&sdk_meta),
                )
            })
        });
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().noise_threshold(0.02);
    targets = criterion_benchmark
);
criterion_main!(benches);
