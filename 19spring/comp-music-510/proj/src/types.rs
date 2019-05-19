use std::error::Error;

pub trait Sound = Iterator<Item=f32> + 'static + Send; 

pub type R<A> = Result<A, Box<Error>>;

pub const SAMPLE_RATE: f64 = 44_100.0;
