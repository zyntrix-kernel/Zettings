use std::collections::BTreeMap;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum HardwareProbe {
    BluetoothAdapterPresent,
    NetworkWirelessPresent,
    AudioOutputPresent,
    AudioInputPresent,
    BatteryPresent,
    DisplayOutputPresent,
    PrinterPresent,
}

pub type ProbeResults = BTreeMap<HardwareProbe, bool>;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct PolkitActionId(pub String);

impl std::fmt::Display for PolkitActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl PolkitActionId {
    pub const ACTION_PREFIX: &'static str = "org.zyntrix.zettings.";

    pub fn validate(&self) -> Result<(), crate::ZettingsError> {
        if self.0.starts_with(Self::ACTION_PREFIX) && self.0.len() > Self::ACTION_PREFIX.len() {
            Ok(())
        } else {
            Err(crate::ZettingsError::InvalidAction {
                action: self.0.clone(),
            })
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CapabilityRequirement {
    None,
    SessionOnly,
    Polkit {
        action: PolkitActionId,
    },
    Hardware {
        probe: HardwareProbe,
    },
    All {
        requirements: Vec<CapabilityRequirement>,
    },
    Any {
        requirements: Vec<CapabilityRequirement>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AvailabilityReason {
    BackendMissing,
    BackendUnreachable,
    ProbeFailed,
    HardwareMissing,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum CapabilityState {
    Available,
    Unavailable { reason: AvailabilityReason },
    RequiresAuth { action: PolkitActionId },
    ReadOnly { explanation: String },
}

#[derive(Clone, Debug, Default)]
pub struct EvalContext {
    pub probes: ProbeResults,
    pub reachable_backends: BTreeMap<String, bool>,
}

impl EvalContext {
    #[must_use]
    pub fn probe_satisfied(&self, probe: &HardwareProbe) -> bool {
        self.probes.get(probe).copied().unwrap_or(false)
    }

    #[must_use]
    pub fn backend_reachable(&self, backend: &str) -> bool {
        self.reachable_backends
            .get(backend)
            .copied()
            .unwrap_or(false)
    }
}

#[must_use]
pub fn evaluate(requirement: &CapabilityRequirement, ctx: &EvalContext) -> CapabilityState {
    match requirement {
        CapabilityRequirement::None | CapabilityRequirement::SessionOnly => {
            CapabilityState::Available
        }
        CapabilityRequirement::Polkit { action } => CapabilityState::RequiresAuth {
            action: action.clone(),
        },
        CapabilityRequirement::Hardware { probe } => match ctx.probes.get(probe) {
            Some(true) => CapabilityState::Available,
            Some(false) => CapabilityState::Unavailable {
                reason: AvailabilityReason::HardwareMissing,
            },
            None => CapabilityState::Unavailable {
                reason: AvailabilityReason::ProbeFailed,
            },
        },
        CapabilityRequirement::All { requirements } => {
            let mut state = CapabilityState::Available;
            for req in requirements {
                state = weakest(state, evaluate(req, ctx));
                if matches!(&state, CapabilityState::Unavailable { .. }) {
                    return state;
                }
            }
            state
        }
        CapabilityRequirement::Any { requirements } => {
            let mut best = CapabilityState::Unavailable {
                reason: AvailabilityReason::ProbeFailed,
            };
            for req in requirements {
                let candidate = evaluate(req, ctx);
                best = strongest(best, candidate);
                if matches!(
                    best,
                    CapabilityState::Available | CapabilityState::RequiresAuth { .. }
                ) {
                    return best;
                }
            }
            best
        }
    }
}

fn rank(state: &CapabilityState) -> u8 {
    match state {
        CapabilityState::Unavailable { .. } => 0,
        CapabilityState::ReadOnly { .. } => 1,
        CapabilityState::RequiresAuth { .. } => 2,
        CapabilityState::Available => 3,
    }
}

fn strongest(a: CapabilityState, b: CapabilityState) -> CapabilityState {
    if rank(&a) >= rank(&b) { a } else { b }
}

fn weakest(a: CapabilityState, b: CapabilityState) -> CapabilityState {
    if rank(&a) <= rank(&b) { a } else { b }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(probes: &[(HardwareProbe, bool)]) -> EvalContext {
        EvalContext {
            probes: probes.iter().copied().collect(),
            reachable_backends: BTreeMap::new(),
        }
    }

    #[test]
    fn hardware_requirement_reflects_probe() {
        let req = CapabilityRequirement::Hardware {
            probe: HardwareProbe::BatteryPresent,
        };
        assert_eq!(
            evaluate(&req, &ctx(&[(HardwareProbe::BatteryPresent, true)])),
            CapabilityState::Available
        );
        assert!(matches!(
            evaluate(&req, &ctx(&[])),
            CapabilityState::Unavailable { .. }
        ));
    }

    #[test]
    fn polkit_maps_to_requires_auth() {
        let req = CapabilityRequirement::Polkit {
            action: PolkitActionId(String::from("org.zyntrix.zettings.timedate.set-ntp")),
        };
        assert!(matches!(
            evaluate(&req, &ctx(&[])),
            CapabilityState::RequiresAuth { .. }
        ));
    }

    #[test]
    fn all_is_strongest_constraint_and_any_picks_best() {
        let all = CapabilityRequirement::All {
            requirements: vec![
                CapabilityRequirement::None,
                CapabilityRequirement::Hardware {
                    probe: HardwareProbe::BatteryPresent,
                },
            ],
        };
        assert!(matches!(
            evaluate(&all, &ctx(&[])),
            CapabilityState::Unavailable { .. }
        ));

        let any = CapabilityRequirement::Any {
            requirements: vec![
                CapabilityRequirement::Hardware {
                    probe: HardwareProbe::PrinterPresent,
                },
                CapabilityRequirement::SessionOnly,
            ],
        };
        assert_eq!(evaluate(&any, &ctx(&[])), CapabilityState::Available);
    }

    #[test]
    fn action_id_validation_enforces_namespace() {
        assert!(
            PolkitActionId(String::from("org.zyntrix.zettings.audio.set-volume"))
                .validate()
                .is_ok()
        );
        assert!(
            PolkitActionId(String::from("org.freedesktop.other"))
                .validate()
                .is_err()
        );
    }
}
