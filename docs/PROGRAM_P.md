# Program P — Portfolio Research

Program P consumes confirmed strategy streams. It does not confirm its own
inputs and it does not turn portfolio optimization output into evidence for the
strategies it optimized.

The portfolio runner accepts `PortfolioComponent`s with immutable strategy and
data identities, then reports correlation and conditional correlation, signal
and strategy overlap, capital allocation, drawdown interaction, regime
diversification, capacity, turnover, liquidity, concentration, risk budgets,
optimization, and stress. Constraints and context permissions are explicit.

The legacy vector stream is descriptive fixture support. Boundary-safe P input
uses aligned timestamps, availability times, source/scope/time-grid identities,
portfolio context permission, and a bound locked strategy confirmation.
Synthetic fixtures exercise two or more strategy streams, missing values,
conditional regimes, and deterministic replay. Unknown, abstained, null, and
contradictory inputs remain visible in reports. No actual research result,
portfolio optimization, or live allocation is run by infrastructure tests.

Portfolio confirmation freezes the component set, scope, code, controls, null,
costs, and multiplicity family before an independently authorized custodian
releases its holdout. A strategy holdout already exposed during portfolio
construction cannot be called untouched.
