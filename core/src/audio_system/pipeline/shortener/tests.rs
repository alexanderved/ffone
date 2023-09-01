use super::*;

use mueue::*;

#[test]
fn test_downsample_int_rate() {
    let (send, _) = unidirectional_queue();
    let shortener = AudioShortener::new(send);

    let data = vec![1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 3];

    let audio = RawAudioBuffer::new(data.clone(), RawAudioFormat::S24BE);
    let downsampled_audio = {
        let first = i32::from_be_bytes([0, 1, 2, 3]);
        let second = i32::from_be_bytes([0, 4, 5, 6]);
        let res = (first + second) / 2;

        let mut data = res.to_be_bytes()[1..4].to_vec();

        let third = i32::from_be_bytes([0, 1, 2, 3]);
        let res = (first + second + third) / 3;
        data.extend_from_slice(&res.to_be_bytes()[1..4]);

        data
    };
    let no_samples = audio.no_samples();
    assert_eq!(
        shortener.downsample(audio, no_samples / 2).as_slice(),
        downsampled_audio
    );

    let audio = RawAudioBuffer::new(data, RawAudioFormat::S24LE);
    let downsampled_audio = {
        let first = i32::from_le_bytes([1, 2, 3, 0]);
        let second = i32::from_le_bytes([4, 5, 6, 0]);
        let res = (first + second) / 2;

        let mut data = res.to_le_bytes()[0..3].to_vec();

        let third = i32::from_le_bytes([1, 2, 3, 0]);
        let res = (first + second + third) / 3;
        data.extend_from_slice(&res.to_le_bytes()[0..3]);

        data
    };
    let no_samples = audio.no_samples();
    assert_eq!(
        shortener.downsample(audio, no_samples / 2).as_slice(),
        downsampled_audio
    );

    let data = vec![1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6];

    let audio = RawAudioBuffer::new(data.clone(), RawAudioFormat::S24BE);
    let downsampled_audio = {
        let first = i32::from_be_bytes([0, 1, 2, 3]);
        let second = i32::from_be_bytes([0, 4, 5, 6]);
        let res = (first + second) / 2;

        let mut data = res.to_be_bytes()[1..4].to_vec();
        data.extend_from_within(0..3);

        data
    };
    let no_samples = audio.no_samples();
    assert_eq!(
        shortener.downsample(audio, no_samples / 2).as_slice(),
        downsampled_audio
    );

    let audio = RawAudioBuffer::new(data, RawAudioFormat::S24LE);
    let downsampled_audio = {
        let first = i32::from_le_bytes([1, 2, 3, 0]);
        let second = i32::from_le_bytes([4, 5, 6, 0]);
        let res = (first + second) / 2;

        let mut data = res.to_le_bytes()[0..3].to_vec();
        data.extend_from_within(0..3);

        data
    };
    let no_samples = audio.no_samples();
    assert_eq!(
        shortener.downsample(audio, no_samples / 2).as_slice(),
        downsampled_audio
    );
}

#[test]
fn test_downsample_real_rate() {
    let (send, _) = unidirectional_queue();
    let shortener = AudioShortener::new(send);

    let data = vec![1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 3];

    let audio = RawAudioBuffer::new(data.clone(), RawAudioFormat::S24BE);
    let downsampled_audio = {
        let first = i32::from_be_bytes([0, 1, 2, 3]);
        let second = i32::from_be_bytes([0, 4, 5, 6]);
        let res = (first + second) / 2;

        let mut data = vec![1, 2, 3];
        data.extend_from_slice(&res.to_be_bytes()[1..4]);
        data.extend_from_within(3..6);

        data
    };
    let no_samples = audio.no_samples();
    assert_eq!(
        shortener.downsample(audio, no_samples * 2 / 3).as_slice(),
        downsampled_audio
    );

    let audio = RawAudioBuffer::new(data, RawAudioFormat::S24LE);
    let downsampled_audio = {
        let first = i32::from_le_bytes([1, 2, 3, 0]);
        let second = i32::from_le_bytes([4, 5, 6, 0]);
        let res = (first + second) / 2;

        let mut data = vec![1, 2, 3];
        data.extend_from_slice(&res.to_le_bytes()[0..3]);
        data.extend_from_within(3..6);

        data
    };
    let no_samples = audio.no_samples();
    assert_eq!(
        shortener.downsample(audio, no_samples * 2 / 3).as_slice(),
        downsampled_audio
    );

    let data = vec![1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6];

    let audio = RawAudioBuffer::new(data.clone(), RawAudioFormat::S24BE);
    let downsampled_audio = {
        let first = i32::from_be_bytes([0, 1, 2, 3]);
        let second = i32::from_be_bytes([0, 4, 5, 6]);
        let res = (first + second) / 2;

        let mut data = vec![1, 2, 3, 4, 5, 6];
        data.extend_from_slice(&res.to_be_bytes()[1..4]);
        data.extend_from_within(6..9);

        data
    };
    let no_samples = audio.no_samples();
    assert_eq!(
        shortener.downsample(audio, no_samples * 2 / 3).as_slice(),
        downsampled_audio
    );

    let audio = RawAudioBuffer::new(data, RawAudioFormat::S24LE);
    let downsampled_audio = {
        let first = i32::from_le_bytes([1, 2, 3, 0]);
        let second = i32::from_le_bytes([4, 5, 6, 0]);
        let res = (first + second) / 2;

        let mut data = vec![1, 2, 3, 4, 5, 6];
        data.extend_from_slice(&res.to_le_bytes()[0..3]);
        data.extend_from_within(6..9);

        data
    };
    let no_samples = audio.no_samples();
    assert_eq!(
        shortener.downsample(audio, no_samples * 2 / 3).as_slice(),
        downsampled_audio
    );
}

#[test]
fn test_discard() {
    let (send, _) = unidirectional_queue();
    let shortener = AudioShortener::new(send);

    let data = vec![1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 3];
    let audio = RawAudioBuffer::new(data.clone(), RawAudioFormat::S24BE);
    let discraded_audio = vec![4, 5, 6, 1, 2, 3];

    assert_eq!(
        shortener.discard(audio, 3).unwrap().as_slice(),
        discraded_audio
    );

    let data = vec![1, 2, 3];
    let audio = RawAudioBuffer::new(data.clone(), RawAudioFormat::S24BE);

    assert!(shortener.discard(audio, 3).is_none());
}
