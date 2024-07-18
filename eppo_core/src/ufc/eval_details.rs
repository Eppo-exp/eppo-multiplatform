use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{AttributeValue, Attributes, Configuration};

use super::{
    eval::AllocationNonMatchReason, eval_visitor::*, Assignment, AssignmentEvent, AssignmentValue,
    Condition, FlagEvaluationError, Rule, Shard, Split, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalFlagDetails {
    pub flag_key: String,
    pub subject_key: String,
    pub subject_attributes: Attributes,
    pub timestamp: DateTime<Utc>,
    /// Details of configuration used for evaluation. None if configuration hasn't been fetched yet.
    pub configuration_details: Option<ConfigurationDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AssignmentValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<FlagEvaluationError>,
    /// Key of the selected variation.
    pub variation_key: Option<String>,
    /// Value of the selected variation. Could be `None` if no variation is selected, or selected
    /// value is absent in configuration (configuration error).
    pub variation_value: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allocations: Vec<EvalAllocationDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationDetails {
    /// Timestamp when configuration was fetched by the SDK.
    pub fetched_at: DateTime<Utc>,
    /// Timestamp when configuration was published by the server.
    pub published_at: DateTime<Utc>,
    /// Environment the configuration belongs to.
    pub environment_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalAllocationDetails {
    pub key: String,
    pub result: EvalAllocationResult,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evaluated_rules: Vec<EvalRuleDetails>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evaluated_splits: Vec<EvalSplitDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvalAllocationResult {
    Unevaluated,
    Matched,
    BeforeStartDate,
    AfterEndDate,
    FailingRules,
    TrafficExposureMiss,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalRuleDetails {
    pub matched: bool,
    pub conditions: Vec<EvalConditionDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalConditionDetails {
    pub condition: Condition,
    pub attribute_value: Option<AttributeValue>,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalSplitDetails {
    pub variation_key: String,
    pub matched: bool,
    pub shards: Vec<EvalShardDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalShardDetails {
    pub matched: bool,
    pub shard: Shard,
    pub shard_value: u64,
}

pub(crate) struct EvalFlagDetailsBuilder {
    flag_key: String,
    subject_key: String,
    subject_attributes: Attributes,
    now: DateTime<Utc>,
    configuration_details: Option<ConfigurationDetails>,

    result: Option<Result<Assignment, FlagEvaluationError>>,

    variation_key: Option<String>,
    variation_value: Option<Value>,

    /// List of allocation keys. Used to sort `allocation_eval_results`.
    allocation_keys_order: Vec<String>,
    allocation_eval_results: HashMap<String, EvalAllocationDetails>,
}

pub(crate) struct EvalAllocationDetailsBuilder<'a> {
    allocation_details: &'a mut EvalAllocationDetails,
    variation_key: &'a mut Option<String>,
}

pub(crate) struct EvalRuleDetailsBuilder<'a> {
    rule_details: &'a mut EvalRuleDetails,
}

pub(crate) struct EvalSplitDetailsBuilder<'a> {
    split_details: &'a mut EvalSplitDetails,
}

impl EvalFlagDetailsBuilder {
    pub fn new(
        flag_key: String,
        subject_key: String,
        subject_attributes: Attributes,
        now: DateTime<Utc>,
    ) -> EvalFlagDetailsBuilder {
        EvalFlagDetailsBuilder {
            flag_key,
            subject_key,
            subject_attributes,
            now,
            configuration_details: None,
            result: None,
            variation_key: None,
            variation_value: None,
            allocation_keys_order: Vec::new(),
            allocation_eval_results: HashMap::new(),
        }
    }

    pub fn build(mut self) -> EvalFlagDetails {
        let (result, error) = match self.result {
            None => (None, None), // this should never happen
            Some(Ok(Assignment { value, event: _ })) => (Some(value), None),
            Some(Err(err)) => (None, Some(err)),
        };
        EvalFlagDetails {
            flag_key: self.flag_key,
            subject_key: self.subject_key,
            subject_attributes: self.subject_attributes,
            timestamp: self.now,
            configuration_details: self.configuration_details,
            result,
            error,
            variation_key: self.variation_key,
            variation_value: self.variation_value,
            allocations: self
                .allocation_keys_order
                .into_iter()
                .map(|key| match self.allocation_eval_results.remove(&key) {
                    Some(details) => details,
                    None => EvalAllocationDetails {
                        key,
                        result: EvalAllocationResult::Unevaluated,
                        evaluated_rules: Vec::new(),
                        evaluated_splits: Vec::new(),
                    },
                })
                .collect(),
        }
    }
}

impl EvalVisitor for EvalFlagDetailsBuilder {
    type AllocationVisitor<'a> = EvalAllocationDetailsBuilder<'a>;

    fn visit_allocation<'a>(
        &'a mut self,
        allocation: &super::Allocation,
    ) -> Self::AllocationVisitor<'a> {
        let result = self
            .allocation_eval_results
            .entry(allocation.key.clone())
            .or_insert(EvalAllocationDetails {
                key: allocation.key.clone(),
                result: EvalAllocationResult::Unevaluated,
                evaluated_rules: Vec::new(),
                evaluated_splits: Vec::new(),
            });
        EvalAllocationDetailsBuilder {
            allocation_details: result,
            variation_key: &mut self.variation_key,
        }
    }

    fn on_configuration(&mut self, configuration: &Configuration) {
        self.configuration_details = Some(ConfigurationDetails {
            fetched_at: configuration.fetched_at,
            published_at: configuration.flags.created_at,
            environment_name: configuration.flags.environment.name.clone(),
        })
    }

    fn on_flag_configuration(&mut self, flag: &super::Flag) {
        self.allocation_keys_order.truncate(0);
        self.allocation_keys_order
            .extend(flag.allocations.iter().map(|it| &it.key).cloned());
    }

    fn on_variation(&mut self, variation: &super::Variation) {
        self.variation_value = Some(variation.value.clone());
    }

    fn on_result(&mut self, result: &Result<Assignment, FlagEvaluationError>) {
        self.result = Some(result.clone());
    }
}

impl<'b> EvalAllocationVisitor for EvalAllocationDetailsBuilder<'b> {
    type RuleVisitor<'a> = EvalRuleDetailsBuilder<'a>
    where
        Self: 'a;

    type SplitVisitor<'a> = EvalSplitDetailsBuilder<'a>
    where
        Self: 'a;

    fn visit_rule<'a>(&'a mut self, _rule: &Rule) -> EvalRuleDetailsBuilder<'a> {
        self.allocation_details
            .evaluated_rules
            .push(EvalRuleDetails {
                matched: false,
                conditions: Vec::new(),
            });
        EvalRuleDetailsBuilder {
            rule_details: self
                .allocation_details
                .evaluated_rules
                .last_mut()
                .expect("we just inserted an element, so there must be last"),
        }
    }

    fn visit_split<'a>(&'a mut self, split: &Split) -> Self::SplitVisitor<'a> {
        self.allocation_details
            .evaluated_splits
            .push(EvalSplitDetails {
                matched: false,
                variation_key: split.variation_key.clone(),
                shards: Vec::new(),
            });
        EvalSplitDetailsBuilder {
            split_details: self
                .allocation_details
                .evaluated_splits
                .last_mut()
                .expect("we just inserted an element, so there must be last"),
        }
    }

    fn on_result(&mut self, result: Result<&Split, AllocationNonMatchReason>) {
        *self.variation_key = result.ok().map(|split| split.variation_key.clone());

        self.allocation_details.result = match result {
            Ok(_) => EvalAllocationResult::Matched,
            Err(AllocationNonMatchReason::BeforeStartDate) => EvalAllocationResult::BeforeStartDate,
            Err(AllocationNonMatchReason::AfterEndDate) => EvalAllocationResult::AfterEndDate,
            Err(AllocationNonMatchReason::FailingRules) => EvalAllocationResult::FailingRules,
            Err(AllocationNonMatchReason::TrafficExposureMiss) => {
                EvalAllocationResult::TrafficExposureMiss
            }
        };
    }
}

impl<'a> EvalRuleVisitor for EvalRuleDetailsBuilder<'a> {
    fn on_condition_eval(
        &mut self,
        condition: &Condition,
        attribute_value: Option<&AttributeValue>,
        result: bool,
    ) {
        self.rule_details.conditions.push(EvalConditionDetails {
            matched: result,
            condition: condition.clone(),
            attribute_value: attribute_value.cloned(),
        });
    }

    fn on_result(&mut self, result: bool) {
        self.rule_details.matched = result;
    }
}

impl<'a> EvalSplitVisitor for EvalSplitDetailsBuilder<'a> {
    fn on_shard_eval(&mut self, shard: &Shard, shard_value: u64, matches: bool) {
        self.split_details.shards.push(EvalShardDetails {
            matched: matches,
            shard: shard.clone(),
            shard_value,
        });
    }

    fn on_result(&mut self, matches: bool) {
        self.split_details.matched = matches;
    }
}
