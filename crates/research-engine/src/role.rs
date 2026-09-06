//! Role and relationship dispatch for Program R.
//!
//! The atlas is an ontology boundary: it selects semantically compatible
//! operations from declared detector roles. It does not inspect observations,
//! outcomes, or prior findings.

use research_contracts::{
    validate_detector_lineage, ContractError, DetectorLineage, DetectorRole, NativeScale,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// The eleven relationship families declared by the R ontology.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum RelationshipOperator {
    Temporal,
    Directional,
    Spatial,
    Nesting,
    StateTransition,
    Lifecycle,
    Regime,
    LeadLag,
    Context,
    Interaction,
    IncrementalInformation,
}

/// Short alias used by callers constructing question specifications.
pub type Operator = RelationshipOperator;

impl RelationshipOperator {
    pub const ALL: [Self; 11] = [
        Self::Temporal,
        Self::Directional,
        Self::Spatial,
        Self::Nesting,
        Self::StateTransition,
        Self::Lifecycle,
        Self::Regime,
        Self::LeadLag,
        Self::Context,
        Self::Interaction,
        Self::IncrementalInformation,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Temporal => "temporal",
            Self::Directional => "directional",
            Self::Spatial => "spatial",
            Self::Nesting => "nesting",
            Self::StateTransition => "state_transition",
            Self::Lifecycle => "lifecycle",
            Self::Regime => "regime",
            Self::LeadLag => "lead_lag",
            Self::Context => "context",
            Self::Interaction => "interaction",
            Self::IncrementalInformation => "incremental_information",
        }
    }

    pub fn allows_direction(self, direction: &str) -> bool {
        match self {
            Self::Lifecycle | Self::Nesting | Self::StateTransition | Self::Regime => {
                direction == "bidirectional"
            }
            _ => true,
        }
    }

    pub const fn compatible_with(self, role: DetectorRole) -> bool {
        use DetectorRole::*;
        match role {
            LifecycleObject => matches!(
                self,
                Self::Temporal | Self::Nesting | Self::Lifecycle | Self::LeadLag | Self::Context
            ),
            StructuralObject => matches!(
                self,
                Self::Temporal
                    | Self::Directional
                    | Self::Spatial
                    | Self::Nesting
                    | Self::LeadLag
                    | Self::Context
                    | Self::Interaction
                    | Self::IncrementalInformation
            ),
            Event => matches!(
                self,
                Self::Temporal
                    | Self::Directional
                    | Self::Spatial
                    | Self::Lifecycle
                    | Self::LeadLag
                    | Self::Context
                    | Self::Interaction
                    | Self::IncrementalInformation
            ),
            StateRegime => matches!(
                self,
                Self::Temporal
                    | Self::StateTransition
                    | Self::Regime
                    | Self::LeadLag
                    | Self::Context
                    | Self::Interaction
                    | Self::IncrementalInformation
            ),
            DirectionalContext => matches!(
                self,
                Self::Temporal
                    | Self::Directional
                    | Self::Spatial
                    | Self::LeadLag
                    | Self::Regime
                    | Self::Context
                    | Self::Interaction
                    | Self::IncrementalInformation
            ),
            QualityInstrumentation => matches!(
                self,
                Self::Temporal | Self::Context | Self::IncrementalInformation
            ),
            NormalizationMeasure => matches!(
                self,
                Self::Temporal
                    | Self::Spatial
                    | Self::Regime
                    | Self::Context
                    | Self::IncrementalInformation
            ),
            TemporalContext => matches!(
                self,
                Self::Temporal | Self::LeadLag | Self::Regime | Self::Context | Self::Interaction
            ),
            DerivedObject => matches!(
                self,
                Self::Temporal
                    | Self::Directional
                    | Self::Spatial
                    | Self::Nesting
                    | Self::StateTransition
                    | Self::Lifecycle
                    | Self::Regime
                    | Self::LeadLag
                    | Self::Context
                    | Self::Interaction
                    | Self::IncrementalInformation
            ),
            CompositeContext => matches!(
                self,
                Self::Temporal
                    | Self::Spatial
                    | Self::Nesting
                    | Self::StateTransition
                    | Self::Regime
                    | Self::LeadLag
                    | Self::Context
                    | Self::Interaction
                    | Self::IncrementalInformation
            ),
        }
    }
}

/// The reusable research questions owned by each semantic role. These names
/// select a template; they do not assert that a detector exhibits the trait.
pub fn templates_for_role(role: DetectorRole) -> &'static [&'static str] {
    use DetectorRole::*;
    match role {
        LifecycleObject => &[
            "formation",
            "persistence",
            "transition",
            "termination",
            "censoring",
            "prospective_behavior",
        ],
        StructuralObject => &["incidence", "magnitude", "recurrence", "nesting"],
        Event => &[
            "incidence",
            "clustering",
            "magnitude",
            "recurrence",
            "prospective_consequence",
        ],
        StateRegime => &["occupancy", "transition", "duration", "persistence"],
        DirectionalContext => &["direction", "lead_lag", "conditional_response"],
        QualityInstrumentation => &["failure", "incidence", "persistence", "contamination"],
        NormalizationMeasure => &["distribution", "stability", "regime_interpretation"],
        TemporalContext => &["distribution", "stability", "regime_interpretation"],
        DerivedObject => &["derivation", "incremental_information", "stability"],
        CompositeContext => &[
            "interaction",
            "incremental_information",
            "regime_interpretation",
        ],
    }
}

/// A frozen detector declaration consumed by the atlas.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetectorDescriptor {
    pub detector_id: String,
    pub roles: Vec<DetectorRole>,
    pub native_scales: BTreeSet<NativeScale>,
    pub derives_from: Vec<String>,
}

impl DetectorDescriptor {
    pub fn new(
        detector_id: impl Into<String>,
        roles: Vec<DetectorRole>,
        native_scales: BTreeSet<NativeScale>,
    ) -> Result<Self, ContractError> {
        let descriptor = Self {
            detector_id: detector_id.into(),
            roles,
            native_scales,
            derives_from: Vec::new(),
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    pub fn with_lineage(mut self, derives_from: Vec<String>) -> Result<Self, ContractError> {
        self.derives_from = derives_from;
        self.validate()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.detector_id.is_empty() || self.roles.is_empty() {
            return Err(ContractError::Invalid(
                "detector requires nonempty ID and roles".into(),
            ));
        }
        if self.native_scales.is_empty() {
            return Err(ContractError::Invalid(
                "detector requires at least one native scale".into(),
            ));
        }
        if self
            .roles
            .iter()
            .enumerate()
            .any(|(i, role)| self.roles[..i].contains(role))
        {
            return Err(ContractError::Invalid(
                "detector roles must be unique".into(),
            ));
        }
        if self
            .derives_from
            .iter()
            .any(|parent| parent == &self.detector_id)
        {
            return Err(ContractError::Invalid(
                "detector cannot derive from itself".into(),
            ));
        }
        if self.derives_from.iter().any(String::is_empty) {
            return Err(ContractError::Invalid(
                "detector lineage IDs cannot be empty".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleDispatch {
    pub detector_id: String,
    pub role: DetectorRole,
    pub operators: Vec<RelationshipOperator>,
    pub templates: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetectorAtlas {
    detectors: BTreeMap<String, DetectorDescriptor>,
}

impl DetectorAtlas {
    pub fn new(detectors: Vec<DetectorDescriptor>) -> Result<Self, ContractError> {
        let mut indexed = BTreeMap::new();
        for detector in detectors {
            detector.validate()?;
            if indexed
                .insert(detector.detector_id.clone(), detector)
                .is_some()
            {
                return Err(ContractError::Invalid("detector IDs must be unique".into()));
            }
        }
        if indexed.is_empty() {
            return Err(ContractError::Invalid("atlas cannot be empty".into()));
        }
        let lineage = indexed
            .values()
            .map(|detector| DetectorLineage {
                detector_id: detector.detector_id.clone(),
                parent_detector_ids: detector.derives_from.clone(),
            })
            .collect::<Vec<_>>();
        validate_detector_lineage(&lineage)?;
        for detector in indexed.values() {
            if detector
                .derives_from
                .iter()
                .any(|parent| !indexed.contains_key(parent))
            {
                return Err(ContractError::Invalid(format!(
                    "lineage parent is absent from atlas: {}",
                    detector.detector_id
                )));
            }
        }
        Ok(Self { detectors: indexed })
    }

    pub fn detector(&self, detector_id: &str) -> Option<&DetectorDescriptor> {
        self.detectors.get(detector_id)
    }

    pub fn detectors(&self) -> impl Iterator<Item = &DetectorDescriptor> {
        self.detectors.values()
    }

    /// Returns true for direct or transitive detector derivation.
    pub fn is_derived_from(&self, detector_id: &str, ancestor_id: &str) -> bool {
        let mut pending = vec![detector_id];
        let mut visited = HashSet::new();
        while let Some(current) = pending.pop() {
            let Some(detector) = self.detector(current) else {
                continue;
            };
            for parent in &detector.derives_from {
                if parent == ancestor_id {
                    return true;
                }
                if visited.insert(parent.as_str()) {
                    pending.push(parent);
                }
            }
        }
        false
    }

    pub fn shares_ancestor(&self, left: &str, right: &str) -> bool {
        fn ancestors<'a>(atlas: &'a DetectorAtlas, id: &'a str, out: &mut HashSet<&'a str>) {
            if let Some(detector) = atlas.detector(id) {
                for parent in &detector.derives_from {
                    if out.insert(parent.as_str()) {
                        ancestors(atlas, parent, out);
                    }
                }
            }
        }
        let mut left_ancestors = HashSet::new();
        let mut right_ancestors = HashSet::new();
        ancestors(self, left, &mut left_ancestors);
        ancestors(self, right, &mut right_ancestors);
        !left_ancestors.is_disjoint(&right_ancestors)
    }

    pub fn operators_for(
        &self,
        detector_id: &str,
    ) -> Result<Vec<RelationshipOperator>, ContractError> {
        let detector = self
            .detector(detector_id)
            .ok_or_else(|| ContractError::Invalid(format!("unknown detector: {detector_id}")))?;
        Ok(RelationshipOperator::ALL
            .into_iter()
            .filter(|operator| {
                detector
                    .roles
                    .iter()
                    .any(|role| operator.compatible_with(role.clone()))
            })
            .collect())
    }

    pub fn dispatch(&self, detector_id: &str) -> Result<Vec<RoleDispatch>, ContractError> {
        let detector = self
            .detector(detector_id)
            .ok_or_else(|| ContractError::Invalid(format!("unknown detector: {detector_id}")))?;
        Ok(detector
            .roles
            .iter()
            .map(|role| RoleDispatch {
                detector_id: detector.detector_id.clone(),
                role: role.clone(),
                operators: RelationshipOperator::ALL
                    .into_iter()
                    .filter(|operator| operator.compatible_with(role.clone()))
                    .collect(),
                templates: templates_for_role(role.clone()).to_vec(),
            })
            .collect())
    }
}
