use research_tape::{validate_source_inventory, NativeScale, SourceInventory};

#[test]
fn committed_inventory_is_complete_without_private_lake() {
    let inventory: SourceInventory = serde_json::from_str(include_str!(
        "../../../instruments/XAUUSD/source_inventory.json"
    ))
    .unwrap();
    validate_source_inventory(&inventory).unwrap();
    assert_eq!(inventory.sources.len(), 8);
    assert_eq!(
        inventory
            .sources
            .iter()
            .filter(|source| matches!(source.scale, NativeScale::Bar(_)))
            .count(),
        7
    );
    assert_eq!(
        inventory
            .sources
            .iter()
            .filter(|source| matches!(source.scale, NativeScale::Tick))
            .count(),
        1
    );
}
