/// Performant stream processing of buffers.

use num::traits::NumAssignOps;
use std::iter::ExactSizeIterator;
use std::cmp::min;

#[derive(Copy, Clone)]
pub enum Op {
    Set,
    Add,
    Mul,
}

pub trait Stream {
    type Item;

    fn len(&self) -> usize;

    /// for each ix in out, out[ix] op= self[ix]
    /// Invariant: out.len <= self.len.
    fn take(&mut self, out: &mut [Self::Item], op: Op, can_set: bool);
}

pub struct IterStream<A: ExactSizeIterator>(A);

impl<N: NumAssignOps, A: ExactSizeIterator<Item=N>> Stream for IterStream<A> {
    type Item = A::Item;

    fn len(&self) -> usize {
        self.0.len()
    }

    fn take(&mut self, out: &mut [Self::Item], op: Op, can_set: bool) {
        match op {
            Op::Set => {
                assert!(!can_set);
                for i in 0..out.len() { out[i] = self.0.next().unwrap(); }
            }
            Op::Add =>
                for i in 0..out.len() { out[i] += self.0.next().unwrap(); }
            Op::Mul =>
                for i in 0..out.len() { out[i] *= self.0.next().unwrap(); }
        }
    }
}

pub struct CombinedStream<A: Stream, B: Stream> {
    a: A,
    op: Op,
    b: B,
}

impl<N: NumAssignOps, A: Stream<Item=N>, B: Stream<Item=N>> Stream for CombinedStream<A, B> {
    type Item = N;

    fn len(&self) -> usize {
        min(self.a.len(), self.b.len())
    }

    fn take(&mut self, out: &mut [Self::Item], op: Op, can_set: bool) {
        self.a.take(out, Op::Set, can_set);
        self.b.take(out, self.op, false);
    }
}

#[cfg(test)]
mod bench {
    extern crate test;
    use test::{Bencher, black_box};

    #[bench]
    fn bench_native_unboxed(b: &mut Bencher) {
        let n = black_box(100000);
        let mut buf = vec![0.0_f32; n];
        b.iter(|| {
            let it1 = 0..n;
            let it2 = 0..n;
            let it3 = it1.zip(it2)
                .map(|(x, y)| ((x as f32 / 10000.0) + (y as f32 / 100000.0)).sin());
            for (i, v) in it3.enumerate() {
                buf[i] = v;
            }
        });
    }

    // Don't seem to be too much of a difference...

    #[bench]
    fn bench_native_boxed(b: &mut Bencher) {
        let n = black_box(100000);
        let mut buf = vec![0.0_f32; n];
        b.iter(|| {
            let it1 = Box::new(0..n);
            let it2 = Box::new(0..n);
            let it3 = it1.zip(it2)
                .map(|(x, y)| ((x as f32 / 10000.0) + (y as f32 / 100000.0)).sin());
            for (i, v) in Box::new(it3).enumerate() {
                buf[i] = v;
            }
        });
    }
}

