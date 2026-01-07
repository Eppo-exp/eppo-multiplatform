use std::sync::Arc;

use crate::hashmap::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;

use cityhasher::CityHasher;
use md5::{Digest, Md5};
use std::hash::Hasher;

use crate::bandits::{
    BanditCategoricalAttributeCoefficient, BanditModelData, BanditNumericAttributeCoefficient,
};
use crate::configuration::BanditHashingAlgorithm;
use crate::error::EvaluationFailure;
use crate::events::{AssignmentEvent, BanditEvent};
use crate::ufc::{Assignment, AssignmentValue, VariationType};
use crate::{Configuration, EvaluationError, Str};
use crate::{ContextAttributes, SdkMetadata};

use super::eval_assignment::get_assignment_with_visitor;
use super::eval_details::EvaluationDetails;
use super::eval_details_builder::EvalDetailsBuilder;
use super::eval_visitor::{EvalBanditVisitor, NoopEvalVisitor};

#[derive(Debug)]
pub(super) struct BanditEvaluationDetails {
    /// Selected action.
    pub(super) action_key: Str,
    pub(super) action_weight: f64,
    /// Distance between best and selected actions' scores.
    pub(super) optimality_gap: f64,
}

#[derive(Clone, Copy)]
struct Action<'a> {
    key: &'a Str,
    attributes: &'a ContextAttributes,
}

/// Result of evaluating a bandit.
#[derive(Debug, Clone, Serialize)]
pub struct BanditResult {
    /// Selected variation from the feature flag.
    pub variation: Str,
    /// Selected action if any.
    pub action: Option<Str>,
    /// Flag assignment event that needs to be logged to analytics storage.
    pub assignment_event: Option<AssignmentEvent>,
    /// Bandit assignment event that needs to be logged to analytics storage.
    pub bandit_event: Option<BanditEvent>,
}

/// Evaluate the specified string feature flag for the given subject. If resulting variation is
/// a bandit, evaluate the bandit to return the action.
pub fn get_bandit_action(
    configuration: Option<&Configuration>,
    flag_key: &str,
    subject_key: &Str,
    subject_attributes: &ContextAttributes,
    actions: &HashMap<Str, ContextAttributes>,
    default_variation: &Str,
    now: DateTime<Utc>,
    sdk_meta: &SdkMetadata,
) -> BanditResult {
    get_bandit_action_with_visitor(
        &mut NoopEvalVisitor,
        configuration,
        flag_key,
        subject_key,
        subject_attributes,
        actions,
        default_variation,
        now,
        sdk_meta,
    )
}

/// Evaluate the specified string feature flag for the given subject. If resulting variation is
/// a bandit, evaluate the bandit to return the action. In addition, return evaluation details.
pub fn get_bandit_action_details(
    configuration: Option<&Configuration>,
    flag_key: &str,
    subject_key: &Str,
    subject_attributes: &ContextAttributes,
    actions: &HashMap<Str, ContextAttributes>,
    default_variation: &Str,
    now: DateTime<Utc>,
    sdk_meta: &SdkMetadata,
) -> (BanditResult, EvaluationDetails) {
    let mut builder = EvalDetailsBuilder::new(
        flag_key.to_owned(),
        subject_key.to_owned(),
        subject_attributes.to_generic_attributes().into(),
        now,
    );
    let result = get_bandit_action_with_visitor(
        &mut builder,
        configuration,
        flag_key,
        subject_key,
        subject_attributes,
        actions,
        default_variation,
        now,
        sdk_meta,
    );
    let details = builder.build();
    (result, details)
}

/// Evaluate the specified string feature flag for the given subject. If resulting variation is
/// a bandit, evaluate the bandit to return the action.
fn get_bandit_action_with_visitor<V: EvalBanditVisitor>(
    visitor: &mut V,
    configuration: Option<&Configuration>,
    flag_key: &str,
    subject_key: &Str,
    subject_attributes: &ContextAttributes,
    actions: &HashMap<Str, ContextAttributes>,
    default_variation: &Str,
    now: DateTime<Utc>,
    sdk_meta: &SdkMetadata,
) -> BanditResult {
    let Some(configuration) = configuration else {
        let result = BanditResult {
            variation: default_variation.clone(),
            action: None,
            assignment_event: None,
            bandit_event: None,
        };
        visitor.on_result(Err(EvaluationFailure::ConfigurationMissing), &result);
        return result;
    };

    visitor.on_configuration(configuration);

    let assignment = get_assignment_with_visitor(
        Some(configuration),
        &mut visitor.visit_assignment(),
        flag_key,
        subject_key,
        &Arc::new(subject_attributes.to_generic_attributes()),
        Some(VariationType::String),
        now,
    )
    .unwrap_or_default()
    .unwrap_or_else(|| Assignment {
        value: AssignmentValue::String(default_variation.clone()),
        event: None,
    });

    let variation = assignment
        .value
        .to_string()
        .expect("flag assignment in bandit evaluation is always a string");

    let Some(bandit_key) = configuration.get_bandit_key(flag_key, &variation) else {
        // It's not a bandit variation, just return it.
        let result = BanditResult {
            variation,
            action: None,
            assignment_event: assignment.event,
            bandit_event: None,
        };
        visitor.on_result(Err(EvaluationFailure::NonBanditVariation), &result);
        return result;
    };

    visitor.on_bandit_key(bandit_key);

    let Some(bandit) = configuration.get_bandit(bandit_key) else {
        // We've evaluated a flag that resulted in a bandit but now we cannot find the bandit
        // configuration and we cannot proceed.
        //
        // This should normally never happen as it means that there's a mismatch between the
        // general UFC config and bandits config.
        log::warn!(target: "eppo", bandit_key; "unable to find bandit configuration");
        let result = BanditResult {
            variation,
            action: None,
            assignment_event: assignment.event,
            bandit_event: None,
        };
        visitor.on_result(
            Err(EvaluationFailure::Error(
                EvaluationError::UnexpectedConfigurationError,
            )),
            &result,
        );
        return result;
    };

    let evaluation = match bandit.model_data.evaluate(
        flag_key,
        subject_key,
        subject_attributes,
        actions.iter(),
        configuration.bandit_hashing_algorithm,
    ) {
        Ok(evaluation) => evaluation,
        Err(err) => {
            // We've evaluated a flag but now bandit evaluation failed. (Likely to user supplying
            // empty actions, or NaN attributes.)
            //
            // Abort evaluation and return default variant.
            let result = BanditResult {
                variation,
                action: None,
                assignment_event: assignment.event,
                bandit_event: None,
            };
            visitor.on_result(Err(err), &result);
            return result;
        }
    };

    let action_attributes = &actions[&evaluation.action_key];
    let bandit_event = BanditEvent {
        flag_key: flag_key.into(),
        bandit_key: bandit_key.clone(),
        subject: subject_key.clone(),
        action: evaluation.action_key.clone(),
        action_probability: evaluation.action_weight,
        optimality_gap: evaluation.optimality_gap,
        model_version: bandit.model_version.clone(),
        timestamp: now.to_rfc3339(),
        subject_numeric_attributes: subject_attributes.numeric.clone(),
        subject_categorical_attributes: subject_attributes.categorical.clone(),
        action_numeric_attributes: action_attributes.numeric.clone(),
        action_categorical_attributes: action_attributes.categorical.clone(),
        meta_data: sdk_meta.into(),
    };

    let result = BanditResult {
        variation,
        action: Some(evaluation.action_key),
        assignment_event: assignment.event,
        bandit_event: Some(bandit_event),
    };
    visitor.on_result(Ok(()), &result);
    return result;
}

/// Trait for hashing in bandit evaluation.
///
/// This trait abstracts the hashing logic for bandit evaluation, allowing different
/// implementations (MD5, CityHash) to be used interchangeably.
trait BanditHasher: Clone {
    /// Create a new hasher pre-initialized with flag_key + "-" + subject_key
    fn new(flag_key: &str, subject_key: &str) -> Self;

    /// Get the selection hash (0.0..1.0) for choosing action based on weights
    fn selection_hash(&self) -> f64;

    /// Compute hash for shuffling a specific action
    fn action_shuffle_hash(&self, action_key: &str) -> u64;
}

/// MD5-based bandit hasher (10k shards, compatible with existing SDKs)
#[derive(Clone)]
struct Md5BanditHasher {
    selection_hash: f64,
    shuffle_ctx: Md5, // flag_key + "-" + subject_key + "-"
}

impl BanditHasher for Md5BanditHasher {
    fn new(flag_key: &str, subject_key: &str) -> Self {
        const TOTAL_SHARDS: u32 = 10_000;
        let mut base_ctx = Md5::new();
        base_ctx.update(flag_key.as_bytes());
        base_ctx.update(b"-");
        base_ctx.update(subject_key.as_bytes());

        // Compute selection hash once
        let selection_hash = {
            let hash = base_ctx.clone().finalize();
            let value = u32::from_be_bytes(hash[0..4].try_into().unwrap());
            (value % TOTAL_SHARDS) as f64 / TOTAL_SHARDS as f64
        };

        // Prepare context for shuffling
        let mut shuffle_ctx = base_ctx;
        shuffle_ctx.update(b"-");

        Md5BanditHasher {
            selection_hash,
            shuffle_ctx,
        }
    }

    fn selection_hash(&self) -> f64 {
        self.selection_hash
    }

    fn action_shuffle_hash(&self, action_key: &str) -> u64 {
        const TOTAL_SHARDS: u32 = 10_000;
        let mut ctx = self.shuffle_ctx.clone();
        ctx.update(action_key.as_bytes());
        let hash = ctx.finalize();
        let value = u32::from_be_bytes(hash[0..4].try_into().unwrap());
        (value % TOTAL_SHARDS) as u64
    }
}

/// CityHash-based bandit hasher (experimental, better performance)
#[derive(Clone)]
struct CityHashBanditHasher {
    selection_hash: f64,
    shuffle_ctx: CityHasher,
}

impl BanditHasher for CityHashBanditHasher {
    fn new(flag_key: &str, subject_key: &str) -> Self {
        let mut base_ctx = CityHasher::new();
        base_ctx.write(flag_key.as_bytes());
        base_ctx.write(b"-");
        base_ctx.write(subject_key.as_bytes());

        // Compute selection hash once
        let selection_hash = {
            let hash = base_ctx.clone().finish();
            hash as u32 as f64 / u32::MAX as f64
        };

        // Prepare context for shuffling
        let mut shuffle_ctx = base_ctx;
        shuffle_ctx.write(b"-");

        CityHashBanditHasher {
            selection_hash,
            shuffle_ctx,
        }
    }

    fn selection_hash(&self) -> f64 {
        self.selection_hash
    }

    fn action_shuffle_hash(&self, action_key: &str) -> u64 {
        let mut ctx = self.shuffle_ctx.clone();
        ctx.write(action_key.as_bytes());
        ctx.finish() as u32 as u64
    }
}

impl BanditModelData {
    // Exported to super, so we can use it in precomputed evaluation.
    pub(super) fn evaluate<'a>(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &ContextAttributes,
        actions: impl Iterator<Item = (&'a Str, &'a ContextAttributes)>,
        hashing_algorithm: BanditHashingAlgorithm,
    ) -> Result<BanditEvaluationDetails, EvaluationFailure> {
        match hashing_algorithm {
            BanditHashingAlgorithm::Md5 => self.evaluate_with_hasher::<Md5BanditHasher>(
                flag_key,
                subject_key,
                subject_attributes,
                actions,
            ),
            BanditHashingAlgorithm::CityHash => self.evaluate_with_hasher::<CityHashBanditHasher>(
                flag_key,
                subject_key,
                subject_attributes,
                actions,
            ),
        }
    }

    fn evaluate_with_hasher<'a, H: BanditHasher>(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &ContextAttributes,
        actions: impl Iterator<Item = (&'a Str, &'a ContextAttributes)>,
    ) -> Result<BanditEvaluationDetails, EvaluationFailure> {
        let hasher = H::new(flag_key, subject_key);

        // Pseudo-random deterministic shuffle of actions. Shuffling is unique per subject, so when
        // weights change slightly, large swatches of subjects are not reassigned from one action to
        // the same other action (instead, if subject is pushed away from an action, it will get
        // assigned to a pseudo-random other action).
        let shuffled_actions = {
            let mut shuffled_actions = actions
                .map(|(key, attributes)| Action { key, attributes })
                .collect::<Vec<_>>();
            // Sort actions by their shard value. Use action key as tie breaker.
            shuffled_actions.sort_by_cached_key(|action| {
                let hash = hasher.action_shuffle_hash(action.key);
                (hash, action.key)
            });
            shuffled_actions
        };

        if shuffled_actions.len() == 0 {
            return Err(EvaluationFailure::NoActionsSuppliedForBandit);
        }

        // action scores, in the same order as shuffled actions
        let scores = shuffled_actions
            .iter()
            .map(|it| self.score_action(*it, subject_attributes))
            .collect::<Vec<_>>();
        debug_assert_eq!(shuffled_actions.len(), scores.len());

        let best = scores
            .iter()
            .enumerate()
            .max_by(|(i, a), (j, b)| {
                f64::total_cmp(a, b).then_with(|| {
                    // In the case of multiple actions getting the same best score, we need to break
                    // the tie deterministically.
                    //
                    // Compare action names next.
                    //
                    // We're reversing the comparison, so that before-ordered name is considered
                    // higher and wins the best score.
                    Ord::cmp(&shuffled_actions[*i].key, &shuffled_actions[*j].key).reverse()
                })
            })
            .map(|(i, s)| (i, *s))
            .expect("shuffled actions, and therefore scores, contain at least one action");

        let weights = self.weigh_actions(&scores, best);
        debug_assert_eq!(shuffled_actions.len(), weights.len());

        let selection_hash = hasher.selection_hash();

        let selected_action = {
            let mut cumulative_weight = 0.0;
            weights
                .iter()
                .position(|weight| {
                    cumulative_weight += *weight;
                    cumulative_weight > selection_hash
                })
                .unwrap_or_else(|| weights.len() - 1)
        };

        let optimality_gap = best.1 - scores[selected_action];

        Ok(BanditEvaluationDetails {
            action_key: shuffled_actions[selected_action].key.to_owned(),
            action_weight: weights[selected_action],
            optimality_gap,
        })
    }

    /// Weigh actions depending on their scores. Higher-scored actions receive more weight, except
    /// best action which receive the remainder weight.
    fn weigh_actions<'a>(
        &self,
        scores: &[f64],
        (best_action, best_score): (usize, f64),
    ) -> Vec<f64> {
        let n_actions = scores.len() as f64;

        let mut weights = scores
            .iter()
            .enumerate()
            .map(|(i, score)| {
                if i == best_action {
                    0.0 // to be overwritten later
                } else {
                    let min_probability = self.action_probability_floor / n_actions;
                    let weight = 1.0 / (n_actions + self.gamma * (best_score - score));
                    f64::max(weight, min_probability)
                }
            })
            .collect::<Vec<_>>();

        weights[best_action] = f64::max(1.0 - weights.iter().sum::<f64>(), 0.0);

        weights
    }

    fn score_action(&self, action: Action, subject_attributes: &ContextAttributes) -> f64 {
        let Some(coefficients) = self.coefficients.get(action.key.as_str()) else {
            return self.default_action_score;
        };

        coefficients.intercept
            + score_attributes(
                &action.attributes,
                &coefficients.action_numeric_coefficients,
                &coefficients.action_categorical_coefficients,
            )
            + score_attributes(
                subject_attributes,
                &coefficients.subject_numeric_coefficients,
                &coefficients.subject_categorical_coefficients,
            )
    }
}

fn score_attributes(
    attributes: &ContextAttributes,
    numeric_coefficients: &[BanditNumericAttributeCoefficient],
    categorical_coefficients: &[BanditCategoricalAttributeCoefficient],
) -> f64 {
    numeric_coefficients
        .into_iter()
        .map(|coef| {
            attributes
                .numeric
                .get(coef.attribute_key.as_str())
                .cloned()
                .map(f64::from)
                // fend against infinite/NaN attributes as they poison the calculation down the line
                .filter(|n| n.is_finite())
                .map(|value| value * coef.coefficient)
                .unwrap_or(coef.missing_value_coefficient)
        })
        .chain(categorical_coefficients.into_iter().map(|coef| {
            attributes
                .categorical
                .get(coef.attribute_key.as_str())
                .and_then(|value| coef.value_coefficients.get(value.to_str().as_ref()))
                .copied()
                .unwrap_or(coef.missing_value_coefficient)
        }))
        .sum()
}

#[cfg(test)]
mod tests {
    use std::fs::{read_dir, File};

    use chrono::Utc;
    use serde::{Deserialize, Serialize};

    use crate::{
        eval::get_bandit_action, ufc::UniversalFlagConfig, Configuration, ContextAttributes,
        SdkMetadata, Str,
    };

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestFile {
        flag: String,
        default_value: Str,
        subjects: Vec<TestSubject>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestSubject {
        subject_key: Str,
        subject_attributes: ContextAttributes,
        actions: Vec<TestAction>,
        assignment: TestAssignment,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestAction {
        action_key: Str,
        #[serde(flatten)]
        attributes: ContextAttributes,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    struct TestAssignment {
        variation: Str,
        action: Option<Str>,
    }

    #[test]
    fn sdk_test_data() {
        let config = UniversalFlagConfig::from_json(
            SdkMetadata {
                name: "test",
                version: "0.1.0",
            },
            std::fs::read("../sdk-test-data/ufc/bandit-flags-v1.json").unwrap(),
        )
        .unwrap();
        let bandits = serde_json::from_reader(
            File::open("../sdk-test-data/ufc/bandit-models-v1.json").unwrap(),
        )
        .unwrap();

        let config = Configuration::from_server_response(config, Some(bandits));

        for entry in read_dir("../sdk-test-data/ufc/bandit-tests/").unwrap() {
            let entry = entry.unwrap();
            println!("Processing test file: {:?}", entry.path());

            if entry
                .file_name()
                .into_string()
                .unwrap()
                .ends_with(".dynamic-typing.json")
            {
                // Not applicable to Rust as it's strongly statically typed.
                continue;
            }

            let test: TestFile = serde_json::from_reader(File::open(entry.path()).unwrap())
                .expect("cannot parse test file");

            for subject in test.subjects {
                print!("test subject {:?}... ", subject.subject_key);

                let actions = subject
                    .actions
                    .into_iter()
                    .map(|x| (x.action_key, x.attributes.into()))
                    .collect();

                let result = get_bandit_action(
                    Some(&config),
                    &test.flag,
                    &subject.subject_key,
                    &subject.subject_attributes.into(),
                    &actions,
                    &test.default_value,
                    Utc::now(),
                    &SdkMetadata {
                        name: "test",
                        version: "0.1.0",
                    },
                );

                assert_eq!(
                    TestAssignment {
                        variation: result.variation,
                        action: result.action
                    },
                    subject.assignment
                );

                println!("ok")
            }
        }
    }
}
