//! Canonical ABI JSON for every embedded contract method.
//!
//! Each constant contains the ABI JSON text for one embedded contract method set.

/// Plasma contract methods (`Fuse`, `CancelFuse`).
pub const PLASMA_DEFINITION: &str = r#"[
  {"type":"function","name":"Fuse","inputs":[{"name":"address","type":"address"}]},
  {"type":"function","name":"CancelFuse","inputs":[{"name":"id","type":"hash"}]}
]"#;

/// Pillar contract methods.
pub const PILLAR_DEFINITION: &str = r#"[
  {"type":"function","name":"Register","inputs":[{"name":"name","type":"string"},{"name":"producerAddress","type":"address"},{"name":"rewardAddress","type":"address"},{"name":"giveBlockRewardPercentage","type":"uint8"},{"name":"giveDelegateRewardPercentage","type":"uint8"}]},
  {"type":"function","name":"RegisterLegacy","inputs":[{"name":"name","type":"string"},{"name":"producerAddress","type":"address"},{"name":"rewardAddress","type":"address"},{"name":"giveBlockRewardPercentage","type":"uint8"},{"name":"giveDelegateRewardPercentage","type":"uint8"},{"name":"publicKey","type":"string"},{"name":"signature","type":"string"}]},
  {"type":"function","name":"Revoke","inputs":[{"name":"name","type":"string"}]},
  {"type":"function","name":"UpdatePillar","inputs":[{"name":"name","type":"string"},{"name":"producerAddress","type":"address"},{"name":"rewardAddress","type":"address"},{"name":"giveBlockRewardPercentage","type":"uint8"},{"name":"giveDelegateRewardPercentage","type":"uint8"}]},
  {"type":"function","name":"Delegate","inputs":[{"name":"name","type":"string"}]},
  {"type":"function","name":"Undelegate","inputs":[]}
]"#;

/// Token contract methods.
pub const TOKEN_DEFINITION: &str = r#"[
  {"type":"function","name":"IssueToken","inputs":[{"name":"tokenName","type":"string"},{"name":"tokenSymbol","type":"string"},{"name":"tokenDomain","type":"string"},{"name":"totalSupply","type":"uint256"},{"name":"maxSupply","type":"uint256"},{"name":"decimals","type":"uint8"},{"name":"isMintable","type":"bool"},{"name":"isBurnable","type":"bool"},{"name":"isUtility","type":"bool"}]},
  {"type":"function","name":"Mint","inputs":[{"name":"tokenStandard","type":"tokenStandard"},{"name":"amount","type":"uint256"},{"name":"receiveAddress","type":"address"}]},
  {"type":"function","name":"Burn","inputs":[]},
  {"type":"function","name":"UpdateToken","inputs":[{"name":"tokenStandard","type":"tokenStandard"},{"name":"owner","type":"address"},{"name":"isMintable","type":"bool"},{"name":"isBurnable","type":"bool"}]}
]"#;

/// Sentinel contract methods (`Register`, `Revoke`).
pub const SENTINEL_DEFINITION: &str = r#"[
  {"type":"function","name":"Register","inputs":[]},
  {"type":"function","name":"Revoke","inputs":[]}
]"#;

/// Swap contract methods (`RetrieveAssets`).
pub const SWAP_DEFINITION: &str = r#"[
  {"type":"function","name":"RetrieveAssets","inputs":[{"name":"publicKey","type":"string"},{"name":"signature","type":"string"}]}
]"#;

/// Stake contract methods (`Stake`, `Cancel`).
pub const STAKE_DEFINITION: &str = r#"[
  {"type":"function","name":"Stake","inputs":[{"name":"durationInSec","type":"int64"}]},
  {"type":"function","name":"Cancel","inputs":[{"name":"id","type":"hash"}]}
]"#;

/// Accelerator contract methods.
pub const ACCELERATOR_DEFINITION: &str = r#"[
  {"type":"function","name":"CreateProject","inputs":[{"name":"name","type":"string"},{"name":"description","type":"string"},{"name":"url","type":"string"},{"name":"znnFundsNeeded","type":"uint256"},{"name":"qsrFundsNeeded","type":"uint256"}]},
  {"type":"function","name":"AddPhase","inputs":[{"name":"id","type":"hash"},{"name":"name","type":"string"},{"name":"description","type":"string"},{"name":"url","type":"string"},{"name":"znnFundsNeeded","type":"uint256"},{"name":"qsrFundsNeeded","type":"uint256"}]},
  {"type":"function","name":"UpdatePhase","inputs":[{"name":"id","type":"hash"},{"name":"name","type":"string"},{"name":"description","type":"string"},{"name":"url","type":"string"},{"name":"znnFundsNeeded","type":"uint256"},{"name":"qsrFundsNeeded","type":"uint256"}]},
  {"type":"function","name":"Donate","inputs":[]},
  {"type":"function","name":"VoteByName","inputs":[{"name":"id","type":"hash"},{"name":"name","type":"string"},{"name":"vote","type":"uint8"}]},
  {"type":"function","name":"VoteByProdAddress","inputs":[{"name":"id","type":"hash"},{"name":"vote","type":"uint8"}]}
]"#;

/// Bridge contract methods.
pub const BRIDGE_DEFINITION: &str = r#"[
  {"type":"function","name":"WrapToken","inputs":[{"name":"networkClass","type":"uint32"},{"name":"chainId","type":"uint32"},{"name":"toAddress","type":"string"}]},
  {"type":"function","name":"UpdateWrapRequest","inputs":[{"name":"id","type":"hash"},{"name":"signature","type":"string"}]},
  {"type":"function","name":"SetNetwork","inputs":[{"name":"networkClass","type":"uint32"},{"name":"chainId","type":"uint32"},{"name":"name","type":"string"},{"name":"contractAddress","type":"string"},{"name":"metadata","type":"string"}]},
  {"type":"function","name":"RemoveNetwork","inputs":[{"name":"networkClass","type":"uint32"},{"name":"chainId","type":"uint32"}]},
  {"type":"function","name":"SetTokenPair","inputs":[{"name":"networkClass","type":"uint32"},{"name":"chainId","type":"uint32"},{"name":"tokenStandard","type":"tokenStandard"},{"name":"tokenAddress","type":"string"},{"name":"bridgeable","type":"bool"},{"name":"redeemable","type":"bool"},{"name":"owned","type":"bool"},{"name":"minAmount","type":"uint256"},{"name":"feePercentage","type":"uint32"},{"name":"redeemDelay","type":"uint32"},{"name":"metadata","type":"string"}]},
  {"type":"function","name":"SetNetworkMetadata","inputs":[{"name":"networkClass","type":"uint32"},{"name":"chainId","type":"uint32"},{"name":"metadata","type":"string"}]},
  {"type":"function","name":"RemoveTokenPair","inputs":[{"name":"networkClass","type":"uint32"},{"name":"chainId","type":"uint32"},{"name":"tokenStandard","type":"tokenStandard"},{"name":"tokenAddress","type":"string"}]},
  {"type":"function","name":"Halt","inputs":[{"name":"signature","type":"string"}]},
  {"type":"function","name":"Unhalt","inputs":[]},
  {"type":"function","name":"Emergency","inputs":[]},
  {"type":"function","name":"ChangeTssECDSAPubKey","inputs":[{"name":"pubKey","type":"string"},{"name":"oldPubKeySignature","type":"string"},{"name":"newPubKeySignature","type":"string"}]},
  {"type":"function","name":"ChangeAdministrator","inputs":[{"name":"administrator","type":"address"}]},
  {"type":"function","name":"ProposeAdministrator","inputs":[{"name":"address","type":"address"}]},
  {"type":"function","name":"SetAllowKeyGen","inputs":[{"name":"allowKeyGen","type":"bool"}]},
  {"type":"function","name":"SetRedeemDelay","inputs":[{"name":"redeemDelay","type":"uint64"}]},
  {"type":"function","name":"SetBridgeMetadata","inputs":[{"name":"metadata","type":"string"}]},
  {"type":"function","name":"UnwrapToken","inputs":[{"name":"networkClass","type":"uint32"},{"name":"chainId","type":"uint32"},{"name":"transactionHash","type":"hash"},{"name":"logIndex","type":"uint32"},{"name":"toAddress","type":"address"},{"name":"tokenAddress","type":"string"},{"name":"amount","type":"uint256"},{"name":"signature","type":"string"}]},
  {"type":"function","name":"RevokeUnwrapRequest","inputs":[{"name":"transactionHash","type":"hash"},{"name":"logIndex","type":"uint32"}]},
  {"type":"function","name":"Redeem","inputs":[{"name":"transactionHash","type":"hash"},{"name":"logIndex","type":"uint32"}]},
  {"type":"function","name":"NominateGuardians","inputs":[{"name":"guardians","type":"address[]"}]},
  {"type":"function","name":"SetOrchestratorInfo","inputs":[{"name":"windowSize","type":"uint64"},{"name":"keyGenThreshold","type":"uint32"},{"name":"confirmationsToFinality","type":"uint32"},{"name":"estimatedMomentumTime","type":"uint32"}]}
]"#;

/// Liquidity contract methods.
pub const LIQUIDITY_DEFINITION: &str = r#"[
  {"type":"function","name":"Update","inputs":[]},
  {"type":"function","name":"Donate","inputs":[]},
  {"type":"function","name":"Fund","inputs":[{"name":"znnReward","type":"uint256"},{"name":"qsrReward","type":"uint256"}]},
  {"type":"function","name":"BurnZnn","inputs":[{"name":"burnAmount","type":"uint256"}]},
  {"type":"function","name":"SetTokenTuple","inputs":[{"name":"tokenStandards","type":"string[]"},{"name":"znnPercentages","type":"uint32[]"},{"name":"qsrPercentages","type":"uint32[]"},{"name":"minAmounts","type":"uint256[]"}]},
  {"type":"function","name":"NominateGuardians","inputs":[{"name":"guardians","type":"address[]"}]},
  {"type":"function","name":"ProposeAdministrator","inputs":[{"name":"address","type":"address"}]},
  {"type":"function","name":"Emergency","inputs":[]},
  {"type":"function","name":"SetIsHalted","inputs":[{"name":"isHalted","type":"bool"}]},
  {"type":"function","name":"LiquidityStake","inputs":[{"name":"durationInSec","type":"int64"}]},
  {"type":"function","name":"CancelLiquidityStake","inputs":[{"name":"id","type":"hash"}]},
  {"type":"function","name":"UnlockLiquidityStakeEntries","inputs":[]},
  {"type":"function","name":"SetAdditionalReward","inputs":[{"name":"znnReward","type":"uint256"},{"name":"qsrReward","type":"uint256"}]},
  {"type":"function","name":"CollectReward","inputs":[]},
  {"type":"function","name":"ChangeAdministrator","inputs":[{"name":"administrator","type":"address"}]}
]"#;

/// Spork contract methods (`CreateSpork`, `ActivateSpork`).
pub const SPORK_DEFINITION: &str = r#"[
  {"type":"function","name":"CreateSpork","inputs":[{"name":"name","type":"string"},{"name":"description","type":"string"}]},
  {"type":"function","name":"ActivateSpork","inputs":[{"name":"id","type":"hash"}]}
]"#;

/// HTLC contract methods.
pub const HTLC_DEFINITION: &str = r#"[
  {"type":"function","name":"Create","inputs":[{"name":"hashLocked","type":"address"},{"name":"expirationTime","type":"int64"},{"name":"hashType","type":"uint8"},{"name":"keyMaxSize","type":"uint8"},{"name":"hashLock","type":"bytes"}]},
  {"type":"function","name":"Reclaim","inputs":[{"name":"id","type":"hash"}]},
  {"type":"function","name":"Unlock","inputs":[{"name":"id","type":"hash"},{"name":"preimage","type":"bytes"}]},
  {"type":"function","name":"DenyProxyUnlock","inputs":[]},
  {"type":"function","name":"AllowProxyUnlock","inputs":[]}
]"#;

/// Methods shared across embedded contracts.
pub const COMMON_DEFINITION: &str = r#"[
  {"type":"function","name":"DepositQsr","inputs":[]},
  {"type":"function","name":"WithdrawQsr","inputs":[]},
  {"type":"function","name":"CollectReward","inputs":[]},
  {"type":"function","name":"Update","inputs":[]},
  {"type":"function","name":"Donate","inputs":[]},
  {"type":"function","name":"VoteByName","inputs":[{"name":"id","type":"hash"},{"name":"name","type":"string"},{"name":"vote","type":"uint8"}]},
  {"type":"function","name":"VoteByProdAddress","inputs":[{"name":"id","type":"hash"},{"name":"vote","type":"uint8"}]}
]"#;

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn is_json_array(definition: &str) -> bool {
        matches!(
            serde_json::from_str::<Value>(definition).ok(),
            Some(Value::Array(_))
        )
    }

    #[test]
    fn every_definition_parses_as_a_json_array() {
        for (name, def) in [
            ("plasma", PLASMA_DEFINITION),
            ("pillar", PILLAR_DEFINITION),
            ("token", TOKEN_DEFINITION),
            ("sentinel", SENTINEL_DEFINITION),
            ("swap", SWAP_DEFINITION),
            ("stake", STAKE_DEFINITION),
            ("accelerator", ACCELERATOR_DEFINITION),
            ("bridge", BRIDGE_DEFINITION),
            ("liquidity", LIQUIDITY_DEFINITION),
            ("spork", SPORK_DEFINITION),
            ("htlc", HTLC_DEFINITION),
            ("common", COMMON_DEFINITION),
        ] {
            assert!(is_json_array(def), "{name} must parse as a JSON array");
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn definitions_contain_all_their_expected_method_names() {
        let cases: &[(&str, &[&str])] = &[
            (PLASMA_DEFINITION, &["Fuse", "CancelFuse"]),
            (
                PILLAR_DEFINITION,
                &[
                    "Register",
                    "RegisterLegacy",
                    "Revoke",
                    "UpdatePillar",
                    "Delegate",
                    "Undelegate",
                ],
            ),
            (
                TOKEN_DEFINITION,
                &["IssueToken", "Mint", "Burn", "UpdateToken"],
            ),
            (SENTINEL_DEFINITION, &["Register", "Revoke"]),
            (SWAP_DEFINITION, &["RetrieveAssets"]),
            (STAKE_DEFINITION, &["Stake", "Cancel"]),
            (
                ACCELERATOR_DEFINITION,
                &[
                    "CreateProject",
                    "AddPhase",
                    "UpdatePhase",
                    "Donate",
                    "VoteByName",
                    "VoteByProdAddress",
                ],
            ),
            (
                BRIDGE_DEFINITION,
                &[
                    "WrapToken",
                    "UpdateWrapRequest",
                    "SetNetwork",
                    "RemoveNetwork",
                    "SetTokenPair",
                    "SetNetworkMetadata",
                    "RemoveTokenPair",
                    "Halt",
                    "Unhalt",
                    "Emergency",
                    "ChangeTssECDSAPubKey",
                    "ChangeAdministrator",
                    "ProposeAdministrator",
                    "SetAllowKeyGen",
                    "SetRedeemDelay",
                    "SetBridgeMetadata",
                    "UnwrapToken",
                    "RevokeUnwrapRequest",
                    "Redeem",
                    "NominateGuardians",
                    "SetOrchestratorInfo",
                ],
            ),
            (
                LIQUIDITY_DEFINITION,
                &[
                    "Update",
                    "Donate",
                    "Fund",
                    "BurnZnn",
                    "SetTokenTuple",
                    "NominateGuardians",
                    "ProposeAdministrator",
                    "Emergency",
                    "SetIsHalted",
                    "LiquidityStake",
                    "CancelLiquidityStake",
                    "UnlockLiquidityStakeEntries",
                    "SetAdditionalReward",
                    "CollectReward",
                    "ChangeAdministrator",
                ],
            ),
            (SPORK_DEFINITION, &["CreateSpork", "ActivateSpork"]),
            (
                HTLC_DEFINITION,
                &[
                    "Create",
                    "Reclaim",
                    "Unlock",
                    "DenyProxyUnlock",
                    "AllowProxyUnlock",
                ],
            ),
            (
                COMMON_DEFINITION,
                &[
                    "DepositQsr",
                    "WithdrawQsr",
                    "CollectReward",
                    "Update",
                    "Donate",
                    "VoteByName",
                    "VoteByProdAddress",
                ],
            ),
        ];
        for (def, names) in cases {
            for name in *names {
                let needle = format!(r#""name":"{name}""#);
                assert!(
                    def.contains(&needle),
                    "definition must declare method {name}"
                );
            }
        }
    }
}
