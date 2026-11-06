use std::{num::NonZeroU32, time::Duration};

pub struct AudioBuffer {
    metadata: AudioBufferMetadata,
    samples: Vec<f32>,
}

pub struct AudioBufferView<'a> {
    metadata: &'a AudioBufferMetadata,
    samples: &'a [f32],
}

pub struct AudioBufferViewMut<'a> {
    metadata: &'a AudioBufferMetadata,
    samples: &'a mut [f32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioBufferMetadata {
    sample_rate: SampleRate,
    frame_count: FrameCount,
    channel_layout: ChannelLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleRate(NonZeroU32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameCount(usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelLayout {
    positions: Vec<ChannelPosition>,
}

#[derive(Debug, PartialEq)]
pub struct ChannelLayoutBuilder {
    positions: Vec<ChannelPosition>,
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum ChannelPosition {
    FrontLeft,
    FrontRight,
    FrontCenter,
    LowFrequency,
    BackLeft,
    BackRight,
    SideLeft,
    SideRight,
    // continue according https://ffmpeg.org/doxygen/7.0/group__lavu__audio__channels.html
    Discrete(u32),
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyChannelLayout,
    DuplicateChannelPosition(ChannelPosition),
    SampleCountOverflow,
    SampleCountMismatch(usize, usize), // expected vs. actual
    InvalidSampleRate,
}

impl AudioBuffer {
    pub fn new(metadata: AudioBufferMetadata) -> Self {
        let samples = vec![0.0; metadata.sample_count()];
        Self { metadata, samples }
    }

    pub fn from_samples(metadata: AudioBufferMetadata, samples: Vec<f32>) -> Result<Self, Error> {
        if samples.len() != metadata.sample_count() {
            Err(Error::SampleCountMismatch(
                metadata.sample_count(),
                samples.len(),
            ))
        } else {
            Ok(Self { metadata, samples })
        }
    }

    pub fn metadata(&self) -> &AudioBufferMetadata {
        &self.metadata
    }

    pub fn channel(&self, index: usize) -> Option<&[f32]> {
        let (start, end) = channel_bounds(&self.metadata, &self.samples, index)?;
        Some(&self.samples[start..end])
    }

    pub fn channel_mut(&mut self, index: usize) -> Option<&mut [f32]> {
        let (start, end) = channel_bounds(&self.metadata, &self.samples, index)?;
        Some(&mut self.samples[start..end])
    }

    pub fn view(&self) -> AudioBufferView<'_> {
        AudioBufferView {
            metadata: &self.metadata,
            samples: &self.samples,
        }
    }

    pub fn view_mut(&mut self) -> AudioBufferViewMut<'_> {
        AudioBufferViewMut {
            metadata: &self.metadata,
            samples: &mut self.samples,
        }
    }
}

impl<'a> AudioBufferView<'a> {
    pub fn metadata(&self) -> &AudioBufferMetadata {
        self.metadata
    }

    pub fn channel(&self, index: usize) -> Option<&[f32]> {
        let (start, end) = channel_bounds(self.metadata, self.samples, index)?;
        Some(&self.samples[start..end])
    }
}

impl<'a> AudioBufferViewMut<'a> {
    pub fn metadata(&self) -> &AudioBufferMetadata {
        self.metadata
    }

    pub fn channel(&self, index: usize) -> Option<&[f32]> {
        let (start, end) = channel_bounds(self.metadata, self.samples, index)?;
        Some(&self.samples[start..end])
    }

    pub fn channel_mut(&mut self, index: usize) -> Option<&mut [f32]> {
        let (start, end) = channel_bounds(self.metadata, self.samples, index)?;
        Some(&mut self.samples[start..end])
    }
}

impl AudioBufferMetadata {
    pub fn new(
        sample_rate: SampleRate,
        frame_count: FrameCount,
        channel_layout: ChannelLayout,
    ) -> Result<Self, Error> {
        let _ = frame_count
            .0
            .checked_mul(channel_layout.channel_count())
            .ok_or(Error::SampleCountOverflow)?;
        Ok(Self {
            sample_rate,
            frame_count,
            channel_layout,
        })
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count.0
    }

    /// number if individual samples, not frames
    pub fn sample_count(&self) -> usize {
        self.frame_count.0 * self.channel_layout.channel_count()
    }

    pub fn duration(&self) -> Duration {
        let seconds = (self.frame_count.get() as u64) / (self.sample_rate.get() as u64);
        let subsecond_frames = (self.frame_count.get() as u64) % (self.sample_rate.get() as u64);
        let nanoseconds = subsecond_frames * 1_000_000_000 / (self.sample_rate.get() as u64);
        Duration::from_secs(seconds).saturating_add(Duration::from_nanos(nanoseconds))
    }

    pub fn channel_layout(&self) -> &ChannelLayout {
        &self.channel_layout
    }
}

impl ChannelLayout {
    pub fn channel_count(&self) -> usize {
        self.positions.len()
    }

    pub fn position(&self, index: usize) -> Option<ChannelPosition> {
        self.positions.get(index).copied()
    }

    pub fn index(&self, position: ChannelPosition) -> Option<usize> {
        self.positions.iter().position(|&x| x == position)
    }

    pub fn positions(&self) -> &[ChannelPosition] {
        &self.positions
    }
}

impl ChannelLayoutBuilder {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
        }
    }

    pub fn mono() -> Self {
        Self {
            positions: vec![ChannelPosition::FrontCenter],
        }
    }

    pub fn stereo() -> Self {
        Self {
            positions: vec![ChannelPosition::FrontLeft, ChannelPosition::FrontRight],
        }
    }

    pub fn with(mut self, channel_position: ChannelPosition) -> Result<Self, Error> {
        if self.positions.contains(&channel_position) {
            return Err(Error::DuplicateChannelPosition(channel_position));
        }
        self.positions.push(channel_position);
        Ok(self)
    }

    pub fn build(self) -> Result<ChannelLayout, Error> {
        if self.positions.is_empty() {
            Err(Error::EmptyChannelLayout)
        } else {
            Ok(ChannelLayout {
                positions: self.positions,
            })
        }
    }
}

impl Default for ChannelLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleRate {
    pub fn new(sample_rate: u32) -> Result<Self, Error> {
        let sample_rate = std::num::NonZero::new(sample_rate).ok_or(Error::InvalidSampleRate)?;
        Ok(Self(sample_rate))
    }

    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl FrameCount {
    pub fn new(frame_count: usize) -> Self {
        Self(frame_count)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

fn channel_bounds(
    metadata: &AudioBufferMetadata,
    samples: &[f32],
    index: usize,
) -> Option<(usize, usize)> {
    if index >= metadata.channel_layout.channel_count() {
        return None;
    }
    let start = metadata.frame_count.0.checked_mul(index)?;
    let end = start.checked_add(metadata.frame_count.0)?;
    if end <= samples.len() {
        Some((start, end))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn audio_buffer() {
        assert_eq!(
            Err(Error::DuplicateChannelPosition(
                ChannelPosition::FrontCenter
            )),
            ChannelLayoutBuilder::mono().with(ChannelPosition::FrontCenter)
        );

        assert_eq!(
            Err(Error::DuplicateChannelPosition(ChannelPosition::FrontLeft)),
            ChannelLayoutBuilder::stereo().with(ChannelPosition::FrontLeft)
        );

        assert_eq!(
            Err(Error::DuplicateChannelPosition(ChannelPosition::FrontRight)),
            ChannelLayoutBuilder::stereo().with(ChannelPosition::FrontRight)
        );

        let empty_buffer = AudioBuffer::new(
            AudioBufferMetadata::new(
                SampleRate::new(1).unwrap(),
                FrameCount(0),
                ChannelLayoutBuilder::mono().build().unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(None, empty_buffer.channel(1));
        assert_eq!(Some(&[][..]), empty_buffer.channel(0));

        let metadata = AudioBufferMetadata::new(
            SampleRate::new(48_000).unwrap(),
            FrameCount(2 * 48_000),
            ChannelLayoutBuilder::mono().build().unwrap(),
        )
        .unwrap();

        let from_samples = AudioBuffer::from_samples(
            AudioBufferMetadata::new(
                SampleRate::new(48_000).unwrap(),
                FrameCount(1),
                ChannelLayoutBuilder::stereo().build().unwrap(),
            )
            .unwrap(),
            vec![0.5, -0.5],
        )
        .unwrap();
        assert_eq!(from_samples.channel(0).unwrap()[0], 0.5);
        assert_eq!(from_samples.channel(1).unwrap()[0], -0.5);

        let mut audio_buffer = AudioBuffer::new(metadata.clone());

        assert_eq!(metadata.duration(), std::time::Duration::from_secs(2));
        assert_eq!(*audio_buffer.metadata(), metadata);

        {
            let audio_buffer_view = audio_buffer.view();
            assert_eq!(*audio_buffer_view.metadata(), metadata);
            assert_eq!(audio_buffer_view.channel(1), None);
            let channel = audio_buffer_view.channel(0).unwrap();
            assert_eq!(channel.len(), 96_000);
            assert_eq!(channel[0], 0.0);
        }

        {
            let mut audio_buffer_view_mut = audio_buffer.view_mut();
            assert_eq!(*audio_buffer_view_mut.metadata(), metadata);
            let channel = audio_buffer_view_mut.channel_mut(0).unwrap();
            assert_eq!(channel.len(), 96_000);
            assert_eq!(channel[0], 0.0);
            channel[0] = -1.0;
        }

        assert_eq!(audio_buffer.channel(1), None);
        let channel = audio_buffer.channel(0).unwrap();
        assert_eq!(channel.len(), 96_000);
        assert_eq!(channel[0], -1.0)
    }
}
