use crate::*;

use std::sync::atomic::{AtomicU64, Ordering};

// Most code in this module comes from macroquad's rand module, with some additions/tweaks.

const DEFAULT_INC: u64 = 1442695040888963407;
const MULTIPLIER: u64 = 6364136223846793005;

static STATE: AtomicU64 = AtomicU64::new(0);

/// Seeds the pseudo-random number generator used by rand()
/// with the value seed.
pub fn srand(seed: u64) {
    STATE.store(0, Ordering::Relaxed);
    rand();
    let oldstate = STATE.load(Ordering::Relaxed);
    STATE.store(oldstate.wrapping_add(seed), Ordering::Relaxed);
    rand();
}

/// returns a pseudo-random number in the range of 0 to u32::MAX.
pub fn rand() -> u32 {
    let oldstate: u64 = STATE.load(Ordering::Relaxed);
    STATE.store(
        oldstate.wrapping_mul(MULTIPLIER).wrapping_add(DEFAULT_INC),
        Ordering::Relaxed,
    );
    let xorshifted: u32 = (((oldstate >> 18) ^ oldstate) >> 27) as u32;
    let rot: u32 = (oldstate >> 59) as u32;
    xorshifted.rotate_right(rot)
}

pub trait RandomRange {
    fn gen_range(low: Self, high: Self) -> Self;
}

macro_rules! impl_random_range{
    ($($ty:ty),*,)=>{
      $(
        impl RandomRange for $ty{
          #[inline]
          fn gen_range(low: Self, high: Self) -> Self {
            let r = rand() as f64 / (u32::MAX as f64 + 1.0);
            let r = low as f64 + (high as f64 - low as f64) * r;
            r as Self
          }
        }
      )*
    }
  }
impl_random_range!(
    f32, f64, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize,
);

pub fn gen_range<T>(low: T, high: T) -> T
where T: RandomRange {
    T::gen_range(low, high)
}

pub struct VecChooseIter<'a, T> {
    source: &'a Vec<T>,
    indices: std::vec::IntoIter<usize>,
}

impl<'a, T> Iterator for VecChooseIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.indices.next().map(|ix| &self.source[ix])
    }
}

pub trait ChooseRandom<T> {
    fn shuffle(&mut self);
    fn choose(&self) -> Option<&T>;
    fn choose_mut(&mut self) -> Option<&mut T>;
    fn choose_multiple(&self, _amount: usize) -> VecChooseIter<T>;
}

impl<T> ChooseRandom<T> for Vec<T> {
    fn shuffle(&mut self) {
        let mut fy = FisherYates::default();

        fy.shuffle(self);
    }

    fn choose(&self) -> Option<&T> {
        let ix = gen_range(0, self.len());
        self.get(ix)
    }

    fn choose_mut(&mut self) -> Option<&mut T> {
        let ix = gen_range(0, self.len());
        self.get_mut(ix)
    }

    fn choose_multiple(&self, amount: usize) -> VecChooseIter<T> {
        let mut indices =
            (0..self.len()).enumerate().map(|(i, _)| i).collect::<Vec<usize>>();

        indices.shuffle();
        indices.resize(amount, 0);

        VecChooseIter { source: self, indices: indices.into_iter() }
    }
}

/// Implementation of Fisher-Yates algorithm.
/// This is modified version of <https://github.com/adambudziak/shuffle/blob/master/src/fy.rs>
#[derive(Debug, Default)]
pub struct FisherYates {
    buffer: [u8; std::mem::size_of::<usize>()],
}

impl FisherYates {
    pub fn shuffle<T>(&mut self, data: &mut [T]) {
        for i in 1..data.len() {
            let j = self.gen_range(i);
            data.swap(i, j);
        }
    }
}

impl FisherYates {
    fn gen_range(&mut self, top: usize) -> usize {
        const USIZE_BYTES: usize = std::mem::size_of::<usize>();
        let bit_width = USIZE_BYTES * 8 - top.leading_zeros() as usize;
        let byte_count = (bit_width - 1) / 8 + 1;
        loop {
            for i in 0..byte_count {
                self.buffer[i] = gen_range(0, 255);
            }
            let result = usize::from_le_bytes(self.buffer);
            let result = result & ((1 << bit_width) - 1);
            if result < top {
                break result;
            }
        }
    }
}

// pub fn random(min: f32, max: f32) -> f32 {
//     // rand::random::<f32>().abs() * (max - min) + min
//     gen_range(min, max)
// }

pub fn random_i32(min: i32, max: i32) -> i32 {
    // rand::thread_rng().gen_range(min..max)
    gen_range(min, max)
}

pub fn random_usize(min: usize, max: usize) -> usize {
    // rand::thread_rng().gen_range(min..max)
    gen_range(min, max)
}

pub fn flip_coin(p: f32) -> bool {
    toss_coin(p)
}

pub fn coin_toss(p: f32) -> bool {
    toss_coin(p)
}

pub fn toss_coin(p: f32) -> bool {
    gen_range(0.0, 1.0) < p
}

pub fn random_angle() -> f32 {
    gen_range(0.0, 2.0 * PI)
}

pub fn random_range(min: f32, max: f32) -> f32 {
    gen_range(min, max)
}

pub fn random_dir() -> Vec2 {
    let angle = gen_range(0.0, std::f32::consts::PI * 2.0);

    Vec2::new(angle.cos(), angle.sin())
}

pub fn random_vec(min: f32, max: f32) -> Vec2 {
    random_dir() * gen_range(min, max)
}

pub fn random_offset(radius: f32) -> Vec2 {
    random_dir() * gen_range(0.0, radius)
}

pub fn random_circle(radius: f32) -> Vec2 {
    random_offset(radius)
}

pub fn random_box(center: Vec2, size: Vec2) -> Vec2 {
    center +
        vec2(
            gen_range(-size.x, size.x) / 2.0,
            gen_range(-size.y, size.y) / 2.0,
        )
}

pub fn random_around(position: Vec2, min: f32, max: f32) -> Vec2 {
    position + random_vec(min, max)
}

pub fn random() -> f32 {
    gen_range(0.0, 1.0)
}

#[test]
fn crash_poop_gen_range() {
    let things = vec![1, 2, 3];

    let mut i: usize = 0;

    for _ in 0..1000000000usize {
        i += 1;
        if things.choose().is_none() {
            panic!("EXITING AFTER {} ITERATIONS", i);
        }
    }
}
