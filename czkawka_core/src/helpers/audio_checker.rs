use std::fs::File;
use std::io;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use symphonia::core::codecs::audio::{AudioDecoderOptions, CODEC_ID_NULL_AUDIO};
use symphonia::core::errors::Error;
use symphonia::core::errors::Error::IoError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;

use crate::common::progress_stop_handler::check_if_stop_received;

pub fn parse_audio_file(file_handler: File, stop_flag: &Arc<AtomicBool>) -> Result<Option<()>, Error> {
    let mss = MediaSourceStream::new(Box::new(file_handler), Default::default());

    let Ok(mut format) = symphonia::default::get_probe().probe(&Hint::new(), mss, Default::default(), Default::default()) else {
        return Err(Error::Unsupported("probe info not available/file not recognized"));
    };

    let Some(track) = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.as_ref().and_then(|p| p.audio()).is_some_and(|p| p.codec != CODEC_ID_NULL_AUDIO))
    else {
        return Err(Error::Unsupported("not supported audio track"));
    };

    let Some(audio_params) = track.codec_params.as_ref().and_then(|p| p.audio()) else {
        return Err(Error::Unsupported("not supported audio track"));
    };

    let Ok(mut decoder) = symphonia::default::get_codecs().make_audio_decoder(audio_params, &AudioDecoderOptions::default()) else {
        return Err(Error::Unsupported("not supported codec"));
    };

    loop {
        if check_if_stop_received(stop_flag) {
            return Ok(None);
        }

        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => return Ok(Some(())),
            Err(Error::ResetRequired) => {
                return Err(Error::ResetRequired);
            }
            Err(err) => {
                if let IoError(ref er) = err {
                    // Catch eof, not sure how to do it properly
                    if er.kind() == io::ErrorKind::UnexpectedEof {
                        return Ok(Some(()));
                    }
                }
                return Err(err);
            }
        };

        decoder.decode(&packet)?;
    }
}
