//! Random numbers (Engine and Project Specification, section 8.3, point 3): the counter-based
//! generator Philox4x32-10 of Salmon, Moraes, Dror and Shaw (2011), so that every draw is a pure
//! function of (seed, the ordinal of the drawing statement within the run, loop pass, draw
//! number) and never of thread scheduling or of what was drawn before. A uniform Float takes 53
//! bits from two 32-bit outputs, as the specification fixes, and every distribution draws by
//! inverse CDF from the uniform stream, so a draw changes only when an algorithm changes, with a
//! note in the known-issues list.

/// One Philox4x32-10 block: four 32-bit words from a 128-bit counter and a 64-bit key, exactly
/// the function of the Random123 library, whose published test vectors the tests hold.
pub fn philox4x32_10(counter: [u32; 4], key: [u32; 2]) -> [u32; 4] {
    const M0: u32 = 0xD251_1F53;
    const M1: u32 = 0xCD9E_8D57;
    const W0: u32 = 0x9E37_79B9;
    const W1: u32 = 0xBB67_AE85;
    let mut ctr = counter;
    let mut key = key;
    for round in 0..10 {
        if round > 0 {
            key[0] = key[0].wrapping_add(W0);
            key[1] = key[1].wrapping_add(W1);
        }
        let product0 = u64::from(M0) * u64::from(ctr[0]);
        let product1 = u64::from(M1) * u64::from(ctr[2]);
        let (hi0, lo0) = ((product0 >> 32) as u32, product0 as u32);
        let (hi1, lo1) = ((product1 >> 32) as u32, product1 as u32);
        ctr = [hi1 ^ ctr[1] ^ key[0], lo1, hi0 ^ ctr[3] ^ key[1], lo0];
    }
    ctr
}

/// A stream of uniform numbers: the seed as the key, the statement's ordinal and the loop pass in
/// the counter's high words, and the draw number in its low words. The second word holds the
/// index of the call within its statement (0 for the first), so that two draws written in one
/// statement are independent streams; a draw number stays below 2^32 blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stream {
    key: [u32; 2],
    ordinal: u32,
    pass: u32,
    call: u32,
    /// The number of 32-bit words handed out so far.
    position: u64,
    block: [u32; 4],
    block_index: u64,
}

impl Stream {
    /// The stream of the statement with ordinal `ordinal` (1 for the first statement of a run; 0
    /// for a `seed:` given to one call, which has a stream of its own wherever it is written) in
    /// loop pass `pass` (0 outside a loop), under `seed`.
    pub fn new(seed: u64, ordinal: u32, pass: u32) -> Stream {
        Stream::for_call(seed, ordinal, pass, 0)
    }

    /// The stream of the `call`-th drawing call (0 for the first) of the statement `ordinal`.
    pub fn for_call(seed: u64, ordinal: u32, pass: u32, call: u32) -> Stream {
        let key = [seed as u32, (seed >> 32) as u32];
        Stream {
            key,
            ordinal,
            pass,
            call,
            position: 0,
            block: philox4x32_10([0, call, ordinal, pass], key),
            block_index: 0,
        }
    }

    /// The next 32-bit word.
    pub fn next_u32(&mut self) -> u32 {
        let block_index = self.position / 4;
        if block_index != self.block_index {
            self.block = philox4x32_10(
                [block_index as u32, self.call, self.ordinal, self.pass],
                self.key,
            );
            self.block_index = block_index;
        }
        let word = self.block[(self.position % 4) as usize];
        self.position += 1;
        word
    }

    /// The next uniform number in [0, 1) with 53 random bits: the top 27 bits of one word and the
    /// top 26 of the next, as the Mersenne Twister's `genrand_res53` combines them.
    pub fn uniform(&mut self) -> f64 {
        let a = f64::from(self.next_u32() >> 5);
        let b = f64::from(self.next_u32() >> 6);
        (a * 67_108_864.0 + b) / 9_007_199_254_740_992.0
    }

    /// The number of uniforms drawn so far.
    pub fn draws(&self) -> u64 {
        self.position / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The known-answer vectors of the Random123 library (kat_vectors, philox4x32-10).
    #[test]
    fn philox_reproduces_the_published_test_vectors() {
        assert_eq!(
            philox4x32_10([0, 0, 0, 0], [0, 0]),
            [0x6627_e8d5, 0xe169_c58d, 0xbc57_ac4c, 0x9b00_dbd8]
        );
        assert_eq!(
            philox4x32_10([u32::MAX; 4], [u32::MAX; 2]),
            [0x408f_276d, 0x41c8_3b0e, 0xa20b_c7c6, 0x6d54_51fd]
        );
        assert_eq!(
            philox4x32_10(
                [0x243f_6a88, 0x85a3_08d3, 0x1319_8a2e, 0x0370_7344],
                [0xa409_3822, 0x299f_31d0]
            ),
            [0xd16c_fe09, 0x94fd_cceb, 0x5001_e420, 0x2412_6ea1]
        );
    }

    #[test]
    fn a_stream_is_a_function_of_its_seed_ordinal_and_position() {
        let mut a = Stream::new(20_261_019, 1, 0);
        let mut b = Stream::new(20_261_019, 1, 0);
        let first: Vec<f64> = (0..100).map(|_| a.uniform()).collect();
        let again: Vec<f64> = (0..100).map(|_| b.uniform()).collect();
        assert_eq!(first, again);
        assert!(first.iter().all(|u| (0.0..1.0).contains(u)));
        // 53 random bits: every uniform is a multiple of 2^-53.
        assert!(
            first
                .iter()
                .all(|u| (u * 9_007_199_254_740_992.0).fract() == 0.0)
        );
        let mut other_statement = Stream::new(20_261_019, 2, 0);
        let mut other_seed = Stream::new(20_261_020, 1, 0);
        let mut other_pass = Stream::new(20_261_019, 1, 1);
        assert_ne!(other_statement.uniform(), first[0]);
        assert_ne!(other_seed.uniform(), first[0]);
        assert_ne!(other_pass.uniform(), first[0]);
        let mut second_call = Stream::for_call(20_261_019, 1, 0, 1);
        assert_ne!(second_call.uniform(), first[0]);
        let mut first_call = Stream::for_call(20_261_019, 1, 0, 0);
        assert_eq!(first_call.uniform(), first[0]);
        assert_eq!(a.draws(), 100);
    }
}
