use std::io::{Read, Seek};

use bytemuck::cast_slice;

mod demuxer;
pub use demuxer::Demuxer;

mod decoder;
pub use decoder::Decoder;

mod resampler;
pub use resampler::{CodecParams, Resampler, ResamplingDecoder, SampleFormat};

pub use ac_ffmpeg::codec::audio::AudioFrame;

// TODO Sample should be bound to SampleFormat.
pub fn resample<RS, Sample>(input: RS, target: CodecParams) -> anyhow::Result<Vec<Sample>>
where
    RS: Read + Seek,
    Sample: Clone + bytemuck::Pod,
{
    let decoder = Decoder::try_from(input)?.resample(target);

    let mut output: Vec<Sample> = vec![];

    for frame in decoder {
        for plane in frame?.planes().iter() {
            output.extend_from_slice(cast_slice::<u8, Sample>(plane.data()));
        }
    }

    Ok(output)
}
