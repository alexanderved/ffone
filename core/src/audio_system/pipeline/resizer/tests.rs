use super::*;

#[test]
fn test_discard() {
    let data = vec![1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 3];
    let audio = RawAudioBuffer::new(data.clone(), RawAudioFormat::S24BE);
    let discraded_audio = vec![4, 5, 6, 1, 2, 3];

    assert_eq!(discard(audio, 2).as_slice(), discraded_audio);
}

#[test]
fn test_downsample_int_rate() {
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
        downsample(audio, no_samples / 2).as_slice(),
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
        downsample(audio, no_samples / 2).as_slice(),
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
        downsample(audio, no_samples / 2).as_slice(),
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
        downsample(audio, no_samples / 2).as_slice(),
        downsampled_audio
    );
}

#[test]
fn test_downsample_real_rate() {
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
        downsample(audio, no_samples * 2 / 3).as_slice(),
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
        downsample(audio, no_samples * 2 / 3).as_slice(),
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
        downsample(audio, no_samples * 2 / 3).as_slice(),
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
        downsample(audio, no_samples * 2 / 3).as_slice(),
        downsampled_audio
    );
}

#[test]
fn test_upsample_int_rate() {
    let data = vec![1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6];
    let audio = RawAudioBuffer::new(data.clone(), RawAudioFormat::S24BE);
    let desired_no_samples = audio.no_samples() * 3 / 2;

    let new_audio = upsample(audio, desired_no_samples);
    println!("{}: {:?}", new_audio.no_samples(), new_audio);
}
