use std::collections::BTreeMap;

pub type Identity = usize;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Symbol(pub Identity, pub Identity);

#[derive(Clone, Copy)]
pub struct IdentityRange {
    pub begin: Identity,
    pub length: usize
}

pub use VecIdentityPool as IdentityPool;



pub struct VecIdentityPool {
    collection: Vec<IdentityRange>
}

impl VecIdentityPool {
    pub fn new() -> Self {
        Self{collection: vec![IdentityRange{begin: 0, length: 0}]}
    }

    pub fn get_ranges(&mut self) -> &Vec<IdentityRange> {
        &self.collection
    }

    pub fn get(&mut self) -> Identity {
        self.collection[0].begin
    }

    pub fn is_full(&mut self) -> bool {
        self.collection.len() == 1 && self.get() == 0
    }

    pub fn remove(&mut self, identity: Identity) -> bool {
        let range_index = match self.collection.binary_search_by(|probe| if probe.begin <= identity { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }) {
            Ok(index) => index,
            Err(index) => index
        };
        if range_index == 0 || range_index > self.collection.len() {
            return false;
        }
        let is_not_last = range_index < self.collection.len();
        let mut range = &mut self.collection[range_index-1];
        if is_not_last && identity >= range.begin+range.length {
            return false;
        }
        if identity == range.begin {
            range.begin += 1;
            if is_not_last {
                range.length -= 1;
                if range.length == 0 {
                    self.collection.remove(range_index-1);
                }
            }
        } else if is_not_last && identity == range.begin+range.length-1 {
            range.length -= 1;
            if range.length == 0 {
                self.collection.remove(range_index-1);
            }
        } else {
            let count = identity-range.begin;
            let prev_begin = range.begin;
            range.begin = identity+1;
            if is_not_last {
                range.length -= 1+count;
            }
            self.collection.insert(range_index-1, IdentityRange{begin: prev_begin, length: count});
        }
        true
    }

    pub fn insert(&mut self, identity: Identity) -> bool {
        let range_index = match self.collection.binary_search_by(|probe| if probe.begin <= identity { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }) {
            Ok(index) => index,
            Err(index) => index
        };
        let merge_prev_range = if range_index > 0 {
            match &self.collection.get(range_index-1) {
                Some(prev_range) => {
                    if range_index == self.collection.len() || identity < prev_range.begin+prev_range.length {
                        return false;
                    }
                    prev_range.begin+prev_range.length == identity
                },
                None => false
            }
        } else { false };
        let merge_next_range = match &self.collection.get(range_index) {
            Some(next_range) => { identity+1 == next_range.begin },
            None => false
        };
        let is_not_last = range_index+1 < self.collection.len();
        if merge_prev_range && merge_next_range {
            let prev_range = self.collection[range_index-1];
            let next_range = &mut self.collection[range_index];
            next_range.begin = prev_range.begin;
            if is_not_last {
                next_range.length += 1+prev_range.length;
            }
            self.collection.remove(range_index-1);
        } else if merge_prev_range {
            let prev_range = &mut self.collection[range_index-1];
            prev_range.length += 1;
        } else if merge_next_range {
            let next_range = &mut self.collection[range_index];
            next_range.begin -= 1;
            if is_not_last {
                next_range.length += 1;
            }
        } else {
            self.collection.insert(range_index, IdentityRange{begin: identity, length: 1});
        }
        true
    }
}


#[allow(dead_code)]
pub struct BTreeIdentityPool {
    collection: BTreeMap<Identity, usize>
}

#[allow(dead_code)]
impl BTreeIdentityPool {
    pub fn new() -> Self {
        let mut result = Self {
            collection: BTreeMap::new()
        };
        result.collection.insert(0, 0);
        result
    }

    pub fn get_ranges(&mut self) -> Vec<IdentityRange> {
        let mut result: Vec<IdentityRange> = Vec::new();
        for (begin, length) in &self.collection {
            result.push(IdentityRange{begin: *begin, length: *length});
        }
        result
    }

    pub fn get(&mut self) -> Identity {
        *self.collection.iter().next().unwrap().0
    }

    pub fn is_full(&mut self) -> bool {
        self.collection.len() == 1 && self.get() == 0
    }

    fn find(&mut self, identity: Identity) -> usize {
        let mut low: usize = 0;
        let mut high: usize = self.collection.len();
        while low < high {
            let mid = (low+high)>>1;
            if *self.collection.iter().nth(mid).unwrap().0 <= identity {
                low = mid+1;
            } else {
                high = mid;
            }
        }
        low
    }

    pub fn remove(&mut self, identity: Identity) -> bool {
        let index = self.find(identity);
        if index == 0 || index > self.collection.len() {
            return false;
        }
        let is_not_last = index < self.collection.len();
        let (range_begin, range_length) = self.collection.iter_mut().nth(index-1).unwrap();
        if is_not_last && identity >= *range_begin+*range_length {
            return false;
        }
        if identity == *range_begin {
            let begin = *range_begin;
            let length = *range_length;
            self.collection.remove(&begin);
            if length != 1 {
                self.collection.insert(begin+1, if is_not_last { length-1 } else { 0 });
            }
        } else if is_not_last && identity == *range_begin+*range_length-1 {
            *range_length -= 1;
            if *range_length == 0 {
                let begin = *range_begin;
                self.collection.remove(&begin);
            }
        } else {
            let first_length = identity-range_begin;
            let last_length = if is_not_last { *range_length-1-first_length } else { 0 };
            *range_length = first_length;
            self.collection.insert(identity+1, last_length);
        }
        true
    }

    pub fn insert(&mut self, identity: Identity) -> bool {
        let index = self.find(identity);
        let is_not_last = index < self.collection.len();
        let merge_prev_range = if index > 0 {
            let (prev_range_begin, prev_range_length) = self.collection.iter().nth(index-1).unwrap();
            if !is_not_last || identity < *prev_range_begin+*prev_range_length {
                return false;
            }
            identity == *prev_range_begin+*prev_range_length
        } else { false };
        let merge_next_range = if is_not_last {
            let (next_range_begin, _next_range_length) = self.collection.iter().nth(index).unwrap();
            identity+1 == *next_range_begin
        } else { false };
        let is_not_last = index+1 < self.collection.len();
        if merge_prev_range && merge_next_range {
            let (next_range_begin, next_range_length) = self.collection.iter().nth(index).unwrap();
            let begin = *next_range_begin;
            let length = *next_range_length;
            self.collection.remove(&begin);
            let (_prev_range_begin, prev_range_length) = self.collection.iter_mut().nth(index-1).unwrap();
            *prev_range_length = if is_not_last { *prev_range_length+1+length } else { 0 };
        } else if merge_prev_range {
            *self.collection.iter_mut().nth(index-1).unwrap().1 += 1;
        } else if merge_next_range {
            let (next_range_begin, next_range_length) = self.collection.iter().nth(index).unwrap();
            let begin = *next_range_begin;
            let length = if is_not_last { *next_range_length+1 } else { 0 };
            self.collection.remove(&begin);
            self.collection.insert(begin-1, length);
        } else {
            self.collection.insert(identity, 1);
        }
        true
    }
}
