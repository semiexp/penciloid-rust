use super::{MAX_VAL, MAX_SUM, Cand, CAND_ALL};

// (length, sum, available vals) -> (imperative vals, allowed vals)
pub struct Dictionary {
    data: Vec<(Cand, Cand)>,
}
pub const IMPOSSIBLE: (Cand, Cand) = (CAND_ALL, Cand(0));

impl Dictionary {
    pub fn at(&self, len: i32, sum: i32, available: Cand) -> (Cand, Cand) {
        self.data[(((len * (MAX_SUM + 1) + sum) << MAX_VAL) | available.0 as i32) as usize]
    }
    pub fn imperative(&self, len: i32, sum: i32, available: Cand) -> Cand {
        self.at(len, sum, available).0
    }
    pub fn allowed(&self, len: i32, sum: i32, available: Cand) -> Cand {
        self.at(len, sum, available).1
    }
    
    pub fn default() -> Dictionary {
        let mut data = vec![IMPOSSIBLE; ((MAX_VAL + 1) * (MAX_SUM + 1) * (1 << MAX_VAL as i32)) as usize];
        for vals in 0..(1 << MAX_VAL) {
            data[vals] = (Cand(0), Cand(0));
        }

        for len in 1..(MAX_VAL + 1) {
            for sum in 1..(MAX_SUM + 1) {
                for vals in 0..(1 << MAX_VAL) {
                    let mut imperative = CAND_ALL;
                    let mut allowed = Cand(0);
                    for i in 1..(MAX_VAL + 1) {
                        if Cand(vals).is_set(i) && sum >= i {
                            let nxt = data[(((len - 1) * (MAX_SUM + 1) + (sum - i) << MAX_VAL) | (vals as i32 ^ (1 << (i - 1)))) as usize];
                            if nxt != IMPOSSIBLE {
                                imperative &= nxt.0 | Cand::singleton(i);
                                allowed |= nxt.1 | Cand::singleton(i);
                            }
                        }
                    }
                    data[(((len * (MAX_SUM + 1) + sum) << MAX_VAL) | (vals as i32)) as usize] = (imperative, allowed);
                }
            }
        }
        Dictionary {
            data: data
        }
    }
    pub fn limited() -> Dictionary {
        let mut data = vec![IMPOSSIBLE; ((MAX_VAL + 1) * (MAX_SUM + 1) * (1 << MAX_VAL as i32)) as usize];
        for vals in 0..(1 << MAX_VAL) {
            data[vals] = (Cand(0), Cand(0));
        }
        for len in 1..(MAX_VAL + 1) {
            for sum in 1..(MAX_SUM + 1) {
                for vals in 0..(1u32 << MAX_VAL) {
                    let vals = Cand(vals);
                    if vals.count_set_cands() < len { continue; }

                    let mut cand = vec![];
                    for i in 1..(MAX_VAL + 1) {
                        if vals.is_set(i) {
                            cand.push(i);
                        }
                    }
                    let mut imperative = Cand(0);
                    let mut allowed = vals;

                    let mut sum_low: i32 = 0;
                    for i in 0..(len - 1)  {
                        sum_low += cand[i as usize];
                    }
                    let high_max = sum - sum_low;
                    if high_max < cand[(len - 1) as usize] { continue; }
                    if high_max < MAX_VAL {
                        allowed = allowed.limit_upper_bound(high_max);
                    }
                    if high_max == cand[(len - 1) as usize] + 1 {
                        allowed = allowed.exclude(high_max - 1);
                    }

                    cand.reverse();
                    let mut sum_high: i32 = 0;
                    for i in 0..(len - 1)  {
                        sum_high += cand[i as usize];
                    }
                    let low_min = sum - sum_high;
                    if low_min > cand[(len - 1) as usize] { continue; }
                    if low_min > 1 {
                        allowed = allowed.limit_lower_bound(low_min);
                    }
                    if low_min == cand[(len - 1) as usize] - 1 {
                        allowed = allowed.exclude(low_min + 1);
                    }
                    data[(((len * (MAX_SUM + 1) + sum) << MAX_VAL) | (vals.0 as i32)) as usize] = (imperative, allowed);
                }
            }
        }
        Dictionary {
            data: data
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_default() {
        let dic = Dictionary::default();

        assert_eq!(dic.at(1,  0, CAND_ALL), IMPOSSIBLE);
        assert_eq!(dic.at(1,  1, CAND_ALL), (Cand(0b000000001), Cand(0b000000001)));
        assert_eq!(dic.at(1,  8, CAND_ALL), (Cand(0b010000000), Cand(0b010000000)));
        assert_eq!(dic.at(1,  9, CAND_ALL), (Cand(0b100000000), Cand(0b100000000)));
        assert_eq!(dic.at(1, 10, CAND_ALL), IMPOSSIBLE);
        assert_eq!(dic.at(2,  2, CAND_ALL), IMPOSSIBLE);
        assert_eq!(dic.at(2,  3, CAND_ALL), (Cand(0b000000011), Cand(0b000000011)));
        assert_eq!(dic.at(2,  4, CAND_ALL), (Cand(0b000000101), Cand(0b000000101)));
        assert_eq!(dic.at(2,  5, CAND_ALL), (Cand(0b000000000), Cand(0b000001111)));
        assert_eq!(dic.at(2, 16, CAND_ALL), (Cand(0b101000000), Cand(0b101000000)));
        assert_eq!(dic.at(2, 17, CAND_ALL), (Cand(0b110000000), Cand(0b110000000)));
        assert_eq!(dic.at(2, 18, CAND_ALL), IMPOSSIBLE);
        assert_eq!(dic.at(3,  7, CAND_ALL), (Cand(0b000001011), Cand(0b000001011)));
        assert_eq!(dic.at(3,  8, CAND_ALL), (Cand(0b000000001), Cand(0b000011111)));
        assert_eq!(dic.at(3, 24, CAND_ALL), (Cand(0b111000000), Cand(0b111000000)));
        assert_eq!(dic.at(4, 10, CAND_ALL), (Cand(0b000001111), Cand(0b000001111)));
        assert_eq!(dic.at(4, 12, CAND_ALL), (Cand(0b000000011), Cand(0b000111111)));
        assert_eq!(dic.at(9, 44, CAND_ALL), IMPOSSIBLE);
        assert_eq!(dic.at(9, 45, CAND_ALL), (Cand(0b111111111), Cand(0b111111111)));
    }

    #[test]
    fn test_dictionary_limited_soundness() {
        let dic_default = Dictionary::default();
        let dic_limited = Dictionary::limited();

        for len in 0..(MAX_VAL + 1) {
            for sum in 1..(MAX_SUM + 1) {
                for vals in 0..(1 << MAX_VAL) {
                    let vals = Cand(vals);
                    let (imperative, allowed) = dic_default.at(len, sum, vals);
                    let (imperative_lim, allowed_lim) = dic_limited.at(len, sum, vals);

                    if (imperative_lim, allowed_lim) == IMPOSSIBLE {
                        assert_eq!((imperative, allowed), IMPOSSIBLE);
                    }
                    assert_eq!(imperative & imperative_lim, imperative_lim);
                    assert_eq!(allowed & allowed_lim, allowed);
                }
            }
        }
    }
}
