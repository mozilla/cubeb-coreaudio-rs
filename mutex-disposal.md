# Defusing the *OwnedCriticalSection*

The *OwnedCriticalSection*(or *ocs*) is a custom *mutex* around *pthread_mutex_t*.
It's better to replace the *ocs* by standard *Rust mutex*.
It can make debugging code easier.

## Lock Usage

### *Cubeb* Context

#### The buffer-frames issue

Changing the *buffer-frame-size* of a device may cause troubles
while another stream is actively using the same device at the same time.
See [here][chg-buf-sz] for more detail.

The current solution is to keep tracking how many streams within a context,
and set the latency/buffer-frames of the first stream to the *global* latency/buffer-frames,
then use this *global* latency/buffer-frames as the *buffer-frame-size* of the current device.
If there are other streams come within the context later,
the latency/buffer-frames of other streams will be overwritten
to the same value of the *global* latency/buffer-frames.

Specifically, the *ocs* is used to avoid the following potential race operations:
- prevent *buffer-frame-size* from being writing in parallel
- avoid counting streams in parallel
- avoid setting *global* latency/buffer-frames in parallel

##### Defects
The *buffer-frame-size* of the device might still be changed
while the other stream is actively using it
if the streams are in different *cubeb context*.

However, once [*audioipc*][audioipc] works are done properly,
we should have only one *cubeb context* in the parent process.

The solution we have now is not ideal.
The reason is that we overwrite the latency/buffer-frames of a stream
if it's not the first stream in the cubeb context.
However, the first stream and the later streams may use different devices,
so the the latency/buffer-frames of a stream that operates on
different device than the first stream's one should not be overwritten.

##### Current Code Flow
The whole stream *initialization*(`AudioUnitContext::stream_init` called by `cubeb_stream_init`)
and *destroying*(`AudioUnitStream::drop/destroy` called by `cubeb_stream_destroy`) are locked by a *ocs*.

When stream is *initialized* or *destroyed* in the *cubeb context*,
the active streams(`context.active_streams`) is *increased* or *decreased*
by `audiounit_increment_active_streams` or `audiounit_decrement_active_streams`.

When the first stream is *initialized*,
the `stream.latency_frames` is set to `context.global_latency_frames` by `audiounit_set_global_latency`.
The `latency_frames` of the later streams are overwritten to the value of `global_latency_frames`,
`stream.latency_frames = stream.context.global_latency_frames`.

The `audiounit_set_buffer_size` is called by every streams,
however, only the first stream works.
The later calls of `audiounit_set_buffer_size` will hit the *early-return* condition that
`new_buffer_frame_size == current_buffer_frame_size`
since all the latencies of the later streams are same as the latency of the first stream.

#### Device collection change
The following global variables for tracking device-collection may be operated in different threads

- {input, output}_device_array
- {input, output}_collection_changed_callback
- {input, output}_collection_changed_user_ptr

so they should be protected by a mutex.

### *Cubeb* Stream
The mutex in cubeb stream is to prevent the stream re-initialization and stream destroy
from being executed at the same time.

The mutex is locked for stream-setup and stream-close.
The stream-setup is called when creating/initializing a stream,
and re-initializeing a stream when the device is switched.
The stream-close is called when stream re-initializeing and stream destroy.

While setting up the stream, we will do some device settings.
Some stream variables will be bound to the devices.
While closing the stream, we will tidy up device settings.
Therefore, the variables related to device settings should be protected by a mutex.

See more details on [cubeb pull 163][cubeb-pull-163].

[chg-buf-sz]: https://cs.chromium.org/chromium/src/media/audio/mac/audio_manager_mac.cc?l=982-989&rcl=0207eefb445f9855c2ed46280cb835b6f08bdb30 "issue on changing buffer size"
[audioipc]: https://github.com/djg/audioipc-2
[cubeb-pull-163]: https://github.com/kinetiknz/cubeb/pull/163