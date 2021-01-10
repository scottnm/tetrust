use rand::Rng;

pub trait RangeRng<T: PartialOrd> {
    fn gen_range(&mut self, lower: T, upper: T) -> T;
}

pub struct ThreadRangeRng {
    rng: rand::rngs::ThreadRng,
}

impl ThreadRangeRng {
    pub fn new() -> ThreadRangeRng {
        ThreadRangeRng {
            rng: rand::thread_rng(),
        }
    }
}

impl<T: PartialOrd + rand::distributions::uniform::SampleUniform> RangeRng<T> for ThreadRangeRng {
    fn gen_range(&mut self, lower: T, upper: T) -> T {
        self.rng.gen_range(lower, upper)
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;

    pub struct SingleValueRangeRng<T: PartialOrd + Copy> {
        value: T,
    }

    pub struct SequenceRangeRng<T: PartialOrd + Copy> {
        next: usize,
        seq: Vec<T>,
    }

    impl<T: PartialOrd + Copy> SingleValueRangeRng<T> {
        pub fn new(value: T) -> SingleValueRangeRng<T> {
            SingleValueRangeRng { value }
        }
    }

    impl<T: PartialOrd + Copy> RangeRng<T> for SingleValueRangeRng<T> {
        fn gen_range(&mut self, lower: T, upper: T) -> T {
            assert!(lower <= self.value);
            assert!(upper > self.value);
            self.value
        }
    }

    impl<T: PartialOrd + Copy> SequenceRangeRng<T> {
        pub fn new(value: &[T]) -> SequenceRangeRng<T> {
            SequenceRangeRng {
                next: 0,
                seq: Vec::from(value),
            }
        }
    }

    impl<T: PartialOrd + Copy> RangeRng<T> for SequenceRangeRng<T> {
        fn gen_range(&mut self, lower: T, upper: T) -> T {
            let value = self.seq[self.next];
            self.next = (self.next + 1) % self.seq.len();

            assert!(lower <= value);
            assert!(upper > value);
            value
        }
    }
}
