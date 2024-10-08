use axum::async_trait;
use codec::{dsp::CrossFader, AudioFrame, Pts};

use super::Mixer;

pub struct SilenceMixer {
    cross_fader: CrossFader,
    ad_segment: bool,
    pts: Pts,
}

impl SilenceMixer {
    pub fn new(cross_fader: CrossFader) -> Self {
        Self {
            cross_fader,
            ad_segment: false,
            pts: Pts::new(2_048, 48_000),
        }
    }

    fn start_ad_segment(&mut self) {
        if !self.ad_segment {
            self.cross_fader.reset();
            self.ad_segment = true;
        }
    }

    fn stop_ad_segment(&mut self) {
        if self.ad_segment {
            self.cross_fader.reset();
            self.ad_segment = false;
        }
    }
}

#[async_trait]
impl Mixer for SilenceMixer {
    async fn push(&mut self, kind: analyzer::ContentKind, frame: &AudioFrame) -> AudioFrame {
        self.pts.update(frame);
        let silence = codec::silence_frame(frame);

        let (fade_out, fade_in) = match kind {
            analyzer::ContentKind::Advertisement => {
                self.start_ad_segment();

                (frame, &silence)
            }
            analyzer::ContentKind::Music
            | analyzer::ContentKind::Talk
            | analyzer::ContentKind::Unknown => {
                self.stop_ad_segment();

                (&silence, frame)
            }
        };

        self.cross_fader
            .apply(fade_out, fade_in)
            .with_pts(self.pts.next())
    }
}

#[cfg(test)]
mod tests {
    use analyzer::ContentKind;
    use codec::dsp::ParabolicCrossFade;

    use crate::routes::play::mixer::silence::CrossFader;
    use crate::routes::play::mixer::tests::{create_frames, pts_seq, SamplesAsVec};

    use super::Mixer;
    use super::SilenceMixer;

    #[tokio::test]
    async fn test_music_to_advertisement() {
        let music = create_frames(20, 1.0);

        let mut sut = SilenceMixer::new(CrossFader::exact::<ParabolicCrossFade>(3));

        let mut output = vec![];

        for frame in music.iter().take(5) {
            output.push(sut.push(ContentKind::Music, frame).await);
        }

        for frame in music.iter().skip(5).take(10) {
            output.push(sut.push(ContentKind::Advertisement, frame).await);
        }

        for frame in music.iter().skip(15) {
            output.push(sut.push(ContentKind::Music, frame).await);
        }

        let samples = output
            .iter()
            .flat_map(|frame| frame.samples_as_vec().into_iter())
            .collect::<Vec<_>>();

        assert_eq!(
            &samples,
            &[
                0.0, 0.25, 1.0, 1.0, 1.0, 1.0, 0.25, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.25, 1.0, 1.0, 1.0
            ]
        );

        let timestamps = output
            .iter()
            .map(codec::AudioFrame::pts)
            .collect::<Vec<_>>();

        assert_eq!(timestamps, pts_seq(20));
    }
}
