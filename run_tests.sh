cargo test --verbose
cargo test test_configure_input -- --ignored --skip test_configure_input_with_zero_latency_frames
cargo test test_configure_output -- --ignored --skip test_configure_output_with_zero_latency_frames
cargo test test_aggregate -- --ignored --test-threads 1
cargo test test_create_blank_aggregate_device -- --ignored

# Manual Tests
# cargo test test_switch_output_device -- --ignored -- nocapture
# cargo test test_add_then_remove_listeners -- --ignored -- nocapture
