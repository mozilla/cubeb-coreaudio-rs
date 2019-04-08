# Defusing the *OwnedCriticalSection*

The *OwnedCriticalSection*(or *ocs*) is a custom *mutex* around *pthread_mutex_t*

## Aim
Replace *OwnedCriticalSection* by standard *Rust mutex*

## Lock Usage

### *Cubeb* Context

#### The buffer-frames issue

Changing the *buffer-frame-size* on a device may cause troubles
while another stream is actively using the same device in parallel.
See [here][chg-buf-sz] for more detail.

The current solution is to keep tracking how many streams within a context,
and set the latency/buffer-frames of the first stream to the *global* latency/buffer-frames,
then use this *global* latency/buffer-frames as the *buffer-frame-size* of the current device.
If there are other streams come within the context later,
the latency/buffer-frames of other streams will be overwritten
to the same value of the *global* latency/buffer-frames.

Specifically, the *ocs* is used to avoid the following potential race operations:
- prvent *buffer-frame-size* from being writing in parallel
- avoid counting streams in parallel
- avoid setting *global* latency/buffer-frames in parallel

##### Note
The *buffer-frame-size* of the device might still be changed
while the other stream is actively using it
if the streams are in different *cubeb context*.

However, once [*audioipc*][audioipc] works are done properly,
we should have only one *cubeb context* in the parent process.

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


### *Cubeb* Stream
All members in the *stream* should be protected by a mutex
since the whole *stream* will be *reset* when the audio device it's using is changed.

[chg-buf-sz]: https://cs.chromium.org/chromium/src/media/audio/mac/audio_manager_mac.cc?l=982-989&rcl=0207eefb445f9855c2ed46280cb835b6f08bdb30 "issue on changing buffer size"
[audioipc]: https://github.com/djg/audioipc-2