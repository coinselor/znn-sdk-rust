# Privileged and state-gated operations

The SDK builds account-block templates for every embedded contract method. Template construction does not authorize publishing.

The node validates balances, plasma, signatures, roles, governance state, timing, and protocol constraints when the block is published.

## Account actions

Accounts can publish these actions when the account has the required balance, plasma, and method input:

- transfer ZNN/QSR or a ZTS token with `AccountBlockTemplate::send`
- stake and cancel own stake entries
- fuse/cancel plasma entries owned by the account where protocol rules allow it
- delegate/undelegate to pillars
- issue/mint/burn/update tokens when the account is the relevant owner and pays fees
- create HTLCs and reclaim/unlock when the account satisfies the HTLC rules
- create accelerator projects/phases and vote where governance rules allow it

## Privileged or governance-gated builders

These builders are part of the protocol/API. Ordinary accounts should expect node rejection unless the account has the required role or the network is in the required state:

- Spork: creating or activating sporks is governance/protocol controlled.
- Bridge administration: setting networks/token pairs, halting/unhalting, changing administrators/TSS keys, guardian nomination, orchestrator info, redeem-delay, metadata, and emergency controls require bridge governance/admin authority.
- Liquidity administration: token tuples, additional rewards, administrators, guardians, halt/emergency controls are role-controlled.
- Pillar/sentinel registration: not admin-only, but requires large protocol-defined ZNN/QSR amounts and valid names/addresses.
- Accelerator voting/payout flow: depends on project phase, pillar voting rights, and protocol state.

## Developer guidance

- Treat builder methods as local serialization helpers, not preflight authorization checks.
- Use read APIs first to discover current state: owner/admin addresses, bridge/liquidity metadata, project status, stake/fusion IDs, etc.
- Call `Zenon::requires_pow` or let `Zenon::send` query plasma before publishing.
- For user-facing apps, display node rejection errors clearly and pre-check obvious constraints such as balance, owner address, and known protocol fees.
- Keep privileged-operation tests mocked unless you control a dedicated testnet environment with the right governance/admin keys.
