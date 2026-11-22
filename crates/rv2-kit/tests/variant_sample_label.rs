use rv2_kit::components::variant_test_card::sample_label;

#[test]
fn sample_label_groups_thousands_and_names_the_sample() {
    assert_eq!(sample_label(1200), "Sample: 1,200");
    assert_eq!(sample_label(987), "Sample: 987");
    assert_eq!(sample_label(1_234_567), "Sample: 1,234,567");
}
