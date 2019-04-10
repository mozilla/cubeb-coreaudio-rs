# Regular Tests
cargo test --verbose
cargo test test_configure_input -- --ignored --skip test_configure_input_with_zero_latency_frames
cargo test test_configure_output -- --ignored --skip test_configure_output_with_zero_latency_frames
cargo test test_aggregate -- --ignored --test-threads 1
cargo test test_create_blank_aggregate_device -- --ignored

# Parallel Tests
cargo test test_parallel_ops_init_streams_in_parallel -- --ignored
cargo test test_parallel_init_streams_in_parallel -- --ignored

# Manual Tests
# cargo test test_switch_output_device -- --ignored --nocapture
# cargo test test_add_then_remove_listeners -- --ignored --nocapture
# cargo test test_device_collection_change -- --ignored --nocapture
