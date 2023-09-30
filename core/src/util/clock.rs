use std::cell::{Cell, UnsafeCell};
use std::default::Default;
use std::sync::Arc;
use std::time::*;
use std::{fmt, iter, ops};

use super::RingBuffer;

#[derive(
    Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct ClockTime(u64);

impl ClockTime {
    pub const NANOS_IN_SEC: u64 = 1_000_000_000;
    pub const MICROS_IN_SEC: u64 = 1_000_000;
    pub const MILLIS_IN_SEC: u64 = 1_000;

    pub const ZERO: Self = Self(0);

    pub const NANOSECOND: Self = Self::from_nanos(1);
    pub const MICROSECOND: Self = Self::from_micros(1);
    pub const MILLIECOND: Self = Self::from_millis(1);
    pub const SECOND: Self = Self::from_secs(1);

    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    pub const fn from_micros(micros: u64) -> Self {
        Self(micros * Self::NANOS_IN_SEC / Self::MICROS_IN_SEC)
    }

    pub const fn from_millis(millis: u64) -> Self {
        Self(millis * Self::NANOS_IN_SEC / Self::MILLIS_IN_SEC)
    }

    pub const fn from_secs(secs: u64) -> Self {
        Self(secs * Self::NANOS_IN_SEC)
    }

    pub const fn from_dur(dur: Duration) -> Self {
        Self(dur.as_nanos() as u64)
    }

    pub const fn as_nanos(&self) -> u64 {
        self.0
    }

    pub const fn as_micros(&self) -> u64 {
        self.0 * Self::MICROS_IN_SEC / Self::NANOS_IN_SEC
    }

    pub const fn as_millis(&self) -> u64 {
        self.0 * Self::MILLIS_IN_SEC / Self::NANOS_IN_SEC
    }

    pub const fn as_secs(&self) -> u64 {
        self.0 / Self::NANOS_IN_SEC
    }

    pub const fn as_dur(&self) -> Duration {
        Duration::from_nanos(self.0)
    }

    pub fn to_master_time(&self, calibration_info: ClockCalibrationInfo) -> Self {
        let slope_num = calibration_info.slope_num;
        let slope_denom = calibration_info.slope_denom;

        let master_time_mean = calibration_info.observation_mean.master_time;
        let slave_time_mean = calibration_info.observation_mean.slave_time;

        (*self - slave_time_mean) * slope_num / slope_denom + master_time_mean
    }

    pub fn to_slave_time(&self, calibration_info: ClockCalibrationInfo) -> Self {
        let slope_num = calibration_info.slope_num;
        let slope_denom = calibration_info.slope_denom;

        let master_time_mean = calibration_info.observation_mean.master_time;
        let slave_time_mean = calibration_info.observation_mean.slave_time;

        (*self - master_time_mean) * slope_denom / slope_num + slave_time_mean
    }

    pub fn saturating_sub(self, rhs: Self) -> Self {
        if rhs > self {
            return Self::ZERO;
        }

        self - rhs
    }
}

impl fmt::Debug for ClockTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hours = self.as_secs() / 3600;
        let minutes = self.as_secs() % 3600 / 60;
        let secs = self.as_secs() % 3600 % 60;
        let nanos = self.as_nanos() - self.as_secs() * Self::NANOS_IN_SEC;

        f.write_fmt(format_args!("{}:{}:{}.{:#09}", hours, minutes, secs, nanos))
    }
}

impl ops::Add for ClockTime {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::AddAssign for ClockTime {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl ops::Sub for ClockTime {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl ops::SubAssign for ClockTime {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl ops::Mul for ClockTime {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl ops::Mul<u64> for ClockTime {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Div for ClockTime {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl ops::Div<u64> for ClockTime {
    type Output = Self;

    fn div(self, rhs: u64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ops::Rem for ClockTime {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl ops::Rem<u64> for ClockTime {
    type Output = Self;

    fn rem(self, rhs: u64) -> Self::Output {
        Self(self.0 % rhs)
    }
}

impl iter::Sum for ClockTime {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| a + b).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClockInfo {
    pub name: String,
}

pub trait Clock {
    fn info(&self) -> ClockInfo;

    fn get_time(&self) -> ClockTime;
}

pub const OBSERVATIONS_INTERVAL: ClockTime = ClockTime::from_millis(100);
pub const MIN_OBSERVATIONS: usize = 4;
pub const MAX_OBSERVATIONS: usize = 32;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ClockObservation {
    pub master_time: ClockTime,
    pub slave_time: ClockTime,
}

impl ClockObservation {
    pub fn new(master_time: ClockTime, slave_time: ClockTime) -> Self {
        Self {
            master_time,
            slave_time,
        }
    }

    pub fn mul_times(&self) -> ClockTime {
        self.master_time * self.slave_time
    }
}

impl ops::Add for ClockObservation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            master_time: self.master_time + rhs.master_time,
            slave_time: self.slave_time + rhs.slave_time,
        }
    }
}

impl ops::Sub for ClockObservation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            master_time: self.master_time - rhs.master_time,
            slave_time: self.slave_time - rhs.slave_time,
        }
    }
}

impl ops::Mul for ClockObservation {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            master_time: self.master_time * rhs.master_time,
            slave_time: self.slave_time * rhs.slave_time,
        }
    }
}

impl ops::Mul<u64> for ClockObservation {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        Self {
            master_time: self.master_time * rhs,
            slave_time: self.slave_time * rhs,
        }
    }
}

impl ops::Div for ClockObservation {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            master_time: self.master_time / rhs.master_time,
            slave_time: self.slave_time / rhs.slave_time,
        }
    }
}

impl ops::Div<u64> for ClockObservation {
    type Output = Self;

    fn div(self, rhs: u64) -> Self::Output {
        Self {
            master_time: self.master_time / rhs,
            slave_time: self.slave_time / rhs,
        }
    }
}

impl iter::Sum for ClockObservation {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| a + b).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockCalibrationInfo {
    pub slope_num: u64,
    pub slope_denom: u64,

    pub observation_mean: ClockObservation,
}

impl ClockCalibrationInfo {
    pub fn new(slope_num: u64, slope_denom: u64, observation_mean: ClockObservation) -> Self {
        Self {
            slope_num,
            slope_denom,
            observation_mean,
        }
    }

    pub fn from_observations<'o, I>(observations: I) -> Self
    where
        I: Iterator<Item = &'o ClockObservation> + ExactSizeIterator + Clone,
    {
        let n = observations.len() as u64;

        let observations_sum = observations.clone().copied().sum::<ClockObservation>();
        let observation_mean = observations_sum / n;

        let slave_time_mean = observation_mean.slave_time;

        let slope_num = observations
            .clone()
            .map(ClockObservation::mul_times)
            .sum::<ClockTime>()
            - observation_mean.mul_times() * n;
        let slope_denom = observations
            .map(|observation| observation.slave_time * observation.slave_time)
            .sum::<ClockTime>()
            - slave_time_mean * slave_time_mean * n;

        Self::new(
            slope_num.as_nanos(),
            slope_denom.as_nanos(),
            observation_mean,
        )
    }
}

impl Default for ClockCalibrationInfo {
    fn default() -> Self {
        Self {
            slope_num: 1,
            slope_denom: 1,
            observation_mean: ClockObservation::default(),
        }
    }
}

pub trait SlaveClock: Clock {
    fn get_master(&self) -> Option<Arc<dyn Clock + Send + Sync>>;
    fn set_master(&self, master: Arc<dyn Clock + Send + Sync>);
    fn unset_master(&self);

    fn record_observation(&self);

    fn get_calibration_info(&self) -> ClockCalibrationInfo;
    fn calibrate(&self, info: ClockCalibrationInfo);

    fn get_master_time(&self) -> Option<ClockTime>;
    fn get_slaved_time(&self) -> ClockTime;
}

pub struct SystemClock(Instant);

impl SystemClock {
    pub fn new() -> Self {
        Self(Instant::now())
    }
}

impl Clock for SystemClock {
    fn info(&self) -> ClockInfo {
        ClockInfo {
            name: String::from("System Clock"),
        }
    }

    fn get_time(&self) -> ClockTime {
        ClockTime::from_dur(self.0.elapsed())
    }
}

pub struct SlavedClock<B> {
    base: B,
    master: UnsafeCell<Option<Arc<dyn Clock + Send + Sync>>>,

    observations: UnsafeCell<RingBuffer<ClockObservation, MAX_OBSERVATIONS>>,
    calibration_info: Cell<ClockCalibrationInfo>,
}

impl<B: Clock> SlavedClock<B> {
    pub fn new(base: B) -> Self {
        Self {
            base,
            master: UnsafeCell::new(None),

            observations: UnsafeCell::new(RingBuffer::new()),
            calibration_info: Cell::new(ClockCalibrationInfo::default()),
        }
    }
}

impl<B: Clock> Clock for SlavedClock<B> {
    fn info(&self) -> ClockInfo {
        ClockInfo {
            name: format!("Slaved {}", self.base.info().name),
        }
    }

    fn get_time(&self) -> ClockTime {
        self.base.get_time()
    }
}

impl<B: Clock> SlaveClock for SlavedClock<B> {
    fn get_master(&self) -> Option<Arc<dyn Clock + Send + Sync>> {
        unsafe { (*self.master.get()).clone() }
    }

    fn set_master(&self, master: Arc<dyn Clock + Send + Sync>) {
        unsafe {
            (*self.master.get()).replace(master);
        }

        self.record_observation();
    }

    fn unset_master(&self) {
        unsafe {
            (*self.master.get()).take();
        }
    }

    fn record_observation(&self) {
        let slave_time = self.get_time();
        let master_time = self.get_master_time().unwrap_or(slave_time);

        let observations = unsafe { &mut *self.observations.get() };
        observations.write(ClockObservation::new(master_time, slave_time));

        if observations.len() >= MIN_OBSERVATIONS {
            let calibration_info = ClockCalibrationInfo::from_observations(observations.iter());
            self.calibrate(calibration_info);
        }
    }

    fn get_calibration_info(&self) -> ClockCalibrationInfo {
        self.calibration_info.get()
    }

    fn calibrate(&self, info: ClockCalibrationInfo) {
        self.calibration_info.set(info);
    }

    fn get_master_time(&self) -> Option<ClockTime> {
        unsafe {
            (*self.master.get())
                .as_ref()
                .map(|master| master.get_time())
        }
    }

    fn get_slaved_time(&self) -> ClockTime {
        self.record_observation();

        self.base
            .get_time()
            .to_master_time(self.get_calibration_info())
    }
}

pub struct Timer {
    start: Instant,
    next: Cell<Instant>,
    interval: ClockTime,
}

impl Timer {
    pub fn new(interval: ClockTime) -> Self {
        let start = Instant::now();

        Self {
            start,
            next: Cell::new(start + interval.as_dur()),
            interval,
        }
    }

    pub fn interval(&self) -> ClockTime {
        self.interval
    }

    pub fn set_interval(&mut self, interval: ClockTime) {
        self.interval = interval;
    }

    pub fn reset(&self) {
        let next = self.next.get();
        let next_elapsed = next.elapsed();
        let next_clock_time = ClockTime::from_dur(next_elapsed);

        self.next
            .set(next + (self.interval - next_clock_time % self.interval).as_dur());
    }

    pub fn is_time_out(&self) -> bool {
        let next = self.next.get();
        let next_elapsed = next.elapsed();

        if next_elapsed >= self.interval.as_dur() {
            let next_clock_time = ClockTime::from_dur(next_elapsed);
            self.next
                .set(next + (self.interval - next_clock_time % self.interval).as_dur());

            return true;
        }

        false
    }
}

impl Clock for Timer {
    fn info(&self) -> ClockInfo {
        ClockInfo {
            name: String::from("Timer"),
        }
    }

    fn get_time(&self) -> ClockTime {
        let time = ClockTime::from_dur(self.start.elapsed());

        time % self.interval
    }
}

#[cfg(no_test)]
mod tests {
    #![allow(dead_code)]

    use std::sync::atomic::*;

    use super::*;

    struct FakeClock(AtomicU64);

    impl FakeClock {
        fn new() -> Self {
            FakeClock(AtomicU64::new(0))
        }
    }

    impl Clock for FakeClock {
        fn info(&self) -> ClockInfo {
            ClockInfo {
                name: String::from("Fake Clock"),
            }
        }

        fn get_time(&self) -> ClockTime {
            ClockTime::from_nanos(self.0.fetch_add(1, Ordering::SeqCst))
        }
    }

    struct FakeClock2(Instant);

    impl FakeClock2 {
        fn new() -> Self {
            FakeClock2(Instant::now())
        }
    }

    impl Clock for FakeClock2 {
        fn info(&self) -> ClockInfo {
            ClockInfo {
                name: String::from("Fake Clock2"),
            }
        }

        fn get_time(&self) -> ClockTime {
            ClockTime::from_dur(self.0.elapsed())
        }
    }

    #[test]
    fn test_clock() {
        let sys_clock = Arc::new(SystemClock::new());
        let fake_clock = SlavedClock::new(FakeClock::new());

        fake_clock.set_master(sys_clock.clone());

        for _ in 0..500 {
            fake_clock.record_observation();

            let master = sys_clock.get_time();
            let slave = fake_clock.get_time();
            let slave_to_master = slave.to_master_time(fake_clock.get_calibration_info());
            let master_to_slave = master.to_slave_time(fake_clock.get_calibration_info());

            println!("Master: {master:?}");
            println!("Slave: {slave:?}");
            println!("Slave to Master: {slave_to_master:?}");
            println!("Master to Slave: {master_to_slave:?}\n");
        }
    }
}
