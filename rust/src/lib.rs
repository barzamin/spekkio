use std::{io::Cursor, sync::Arc};

use rustfft::{Fft, FftPlanner};
use symphonia::core::{
    audio::{AudioBuffer, AudioSpec, GenericAudioBufferRef},
    codecs::{audio::{AudioDecoder, AudioDecoderOptions}, registry::CodecRegistry, CodecParameters},
    errors::Error as SymphoniaError,
    formats::{probe::Hint, FormatOptions, FormatReader, TrackType},
    io::{MediaSource, MediaSourceStream},
    meta::MetadataOptions,
};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
}

pub fn hann_window(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| 2.0 * core::f32::consts::PI * (i as f32) / ((n - 1) as f32))
        .map(|phi| 0.5 * (1.0 - phi.cos()))
        .collect()
}

pub struct Analyzer {
    window_size: usize,
    fft_planner: FftPlanner<f32>,
    fft: Arc<dyn Fft<f32>>,
    window_table: Vec<f32>,
}

impl Analyzer {
    pub fn new(window_size: usize) -> Self {
        let mut fft_planner = FftPlanner::new();
        let fft = fft_planner.plan_fft_forward(window_size);

        let window_table = hann_window(window_size);

        Self {
            window_size,
            fft_planner,
            fft,
            window_table,
        }
    }
}

pub struct SymphoniaDecoder<'a> {
    codec_registry: CodecRegistry,
    // format_reader - get raw packets
    // decoder - decode those packets into GenericAudioBufferRef
    format_reader: Box<dyn FormatReader + 'a>,
    decoder: Box<dyn AudioDecoder>,
    // heuristically chosen track (in multi-track files); we only decode packets with this track id
    track_id: u32,
}


impl<'a> SymphoniaDecoder<'a> {
    pub fn new(buf: &'a [u8], mime_hint: Option<&str>) -> Self {
        let mut codec_registry = CodecRegistry::new();
        symphonia::default::register_enabled_codecs(&mut codec_registry);

        let mut hint = Hint::new();
        if let Some(mime_hint) = mime_hint {
            hint.mime_type(mime_hint);
        }

        let cursor = Cursor::new(buf);
        let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

        let probe = symphonia::default::get_probe();
        let fmt_opts: FormatOptions = Default::default();
        let meta_opts: MetadataOptions = Default::default();
        let mut format_reader = probe
            .probe(&hint, mss, fmt_opts, meta_opts)
            .expect("probe error");

        // assume first (decodable) track is what we're looking for
        // let track = format_reader
        let track = format_reader
            .default_track(TrackType::Audio)
            .expect("cant find a default audio track");

        let decode_opts: AudioDecoderOptions = Default::default();
        let mut decoder = codec_registry
            .make_audio_decoder(
                track
                    .codec_params
                    .as_ref()
                    .expect("codec parameters")
                    .audio()
                    .unwrap(),
                &decode_opts,
            )
            .expect("cant make audio decoder/unsupported codec");

        let track_id = track.id;

        Self {
            codec_registry,
            format_reader,
            decoder,
            track_id,
        }
    }

    fn next_chunk(&mut self) -> Result<Option<GenericAudioBufferRef<'_>>, SymphoniaError> {
        loop { // loop until we find a packet with the correct track_id
            let packet = match self.format_reader.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => return Ok(None), // end of stream
                Err(SymphoniaError::ResetRequired) => unimplemented!("lol"),
                Err(err) => return Err(err),
            };

            // consume metadata
            while !self.format_reader.metadata().is_latest() {
                self.format_reader.metadata().pop();
            }

            // only decode track of interest
            if packet.track_id() != self.track_id {
                continue; // go back for another packet
            }

            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    return Ok(Some(decoded))
                }
                Err(SymphoniaError::IoError(_)) => {
                    panic!("cant decode packet: io error");
                }
                Err(SymphoniaError::DecodeError(_)) => {
                    panic!("cant decode packet: invalid data");
                }
                Err(err) => {
                    panic!("unrecoverable error: {}", err);
                }
            }
        }
    }
}
