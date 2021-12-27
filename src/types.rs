use std::error::Error;

pub trait Sound = SoundRef + 'static;
pub trait SoundRef = Iterator<Item=f32>;

pub type R<A> = Result<A, Box<dyn Error>>;

pub const SAMPLE_RATE: f64 = 44_100.0;
