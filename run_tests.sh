# Display backtrace for debugging
export RUST_BACKTRACE=1

# Run tests in the sub crate
# Run the tests by `cargo * -p <SUB_CRATE>` if it's possible. By doing so, the duplicate compiling
# between this crate and the <SUB_CRATE> can be saved. The compiling for <SUB_CRATE> can be reused
# when running `cargo *` with this crate.
# -------------------------------------------------------------------------------------------------
SUB_CRATE="coreaudio-sys-utils"

# Format check
# `cargo fmt -p *` is only usable in workspaces, so a workaround is to enter to the sub crate
# and then exit from it.
cd $SUB_CRATE
cargo fmt --all -- --check
cd ..

# Lints check
cargo clippy -p $SUB_CRATE -- -D warnings

# Regular Tests
cargo test -p $SUB_CRATE

# Run tests in the main crate
# -------------------------------------------------------------------------------------------------
# Format check
cargo fmt --all -- --check

# Lints check
cargo clippy -- -D warnings

# Regular Tests
cargo test --verbose
cargo test test_configure_output -- --ignored
cargo test test_aggregate -- --ignored --test-threads=1

# Parallel Tests
cargo test test_parallel -- --ignored --nocapture --test-threads=1

# Device-changed Tests
cargo test test_switch_device -- --ignored --nocapture
cargo test test_plug_and_unplug_device -- --ignored --nocapture
# cargo test test_register_device_changed_callback -- --ignored --nocapture --test-threads=1
cargo test test_register_device_changed_callback_to_check_default_device_changed_input -- --ignored --nocapture
cargo test test_register_device_changed_callback_to_check_default_device_changed_output -- --ignored --nocapture
cargo test test_register_device_changed_callback_to_check_default_device_changed_duplex -- --ignored --nocapture
cargo test test_register_device_changed_callback_to_check_input_alive_changed_input -- --ignored --nocapture
cargo test test_register_device_changed_callback_to_check_input_alive_changed_duplex -- --ignored --nocapture

cargo test test_destroy_input_stream_after_unplugging_a_nondefault_input_device -- --ignored --nocapture
cargo test test_destroy_input_stream_after_unplugging_a_default_input_device -- --ignored --nocapture
# FIXIT: The following test will hang since we don't monitor the alive status of the output device
# cargo test test_destroy_output_stream_after_unplugging_a_nondefault_output_device -- --ignored --nocapture
cargo test test_destroy_output_stream_after_unplugging_a_default_output_device -- --ignored --nocapture
cargo test test_destroy_duplex_stream_after_unplugging_a_nondefault_input_device -- --ignored --nocapture
cargo test test_destroy_duplex_stream_after_unplugging_a_default_input_device -- --ignored --nocapture
# FIXIT: The following test will hang since we don't monitor the alive status of the output device
# cargo test test_destroy_duplex_stream_after_unplugging_a_nondefault_output_device -- --ignored --nocapture
cargo test test_destroy_duplex_stream_after_unplugging_a_default_output_device -- --ignored --nocapture

cargo test test_reinit_input_stream_by_unplugging_a_nondefault_input_device -- --ignored --nocapture
cargo test test_reinit_input_stream_by_unplugging_a_default_input_device -- --ignored --nocapture
# FIXIT: The following test will hang since we don't monitor the alive status of the output device
# cargo test test_reinit_output_stream_by_unplugging_a_nondefault_output_device -- --ignored --nocapture
cargo test test_reinit_output_stream_by_unplugging_a_default_output_device -- --ignored --nocapture
cargo test test_reinit_duplex_stream_by_unplugging_a_nondefault_input_device -- --ignored --nocapture
cargo test test_reinit_duplex_stream_by_unplugging_a_default_input_device -- --ignored --nocapture
# FIXIT: The following test will hang since we don't monitor the alive status of the output device
# cargo test test_reinit_duplex_stream_by_unplugging_a_nondefault_output_device -- --ignored --nocapture
cargo test test_reinit_duplex_stream_by_unplugging_a_default_output_device -- --ignored --nocapture

# Manual Tests
# cargo test test_switch_output_device -- --ignored --nocapture
# cargo test test_add_then_remove_listeners -- --ignored --nocapture
# cargo test test_device_collection_change -- --ignored --nocapture
# cargo test test_stream_tester -- --ignored --nocapture
