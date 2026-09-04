use research_contracts::{parse_and_validate_contract, validate_context_available};

#[test]
fn example_contract_is_valid() {
    let json = include_str!("../../../templates/experiment_contract.example.json");
    assert!(parse_and_validate_contract(json).is_ok());
}

#[test]
fn future_context_is_rejected() {
    assert!(validate_context_available(100, 101).is_err());
}
