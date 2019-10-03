use std::collections::HashMap;
use std::collections::HashSet;
use std::cell::RefCell;
#[path="bitops.rs"]
mod bitops;

pub type Identity = usize;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Symbol(pub Identity, pub Identity);

pub type Triple = [Symbol; 3];


#[derive(Clone, Copy)]
pub struct IdentityRange {
    begin: Identity,
    length: usize
}

pub struct IdentityPool(Vec<IdentityRange>);

impl IdentityPool {
    pub fn new() -> IdentityPool {
        IdentityPool{0: vec![IdentityRange{begin: 0, length: 0}]}
    }

    pub fn get(&mut self) -> Identity {
        self.0[0].begin
    }

    pub fn remove(&mut self, identity: Identity) -> bool {
        let collection = &mut self.0;
        let range_index = match collection.binary_search_by(|probe| if probe.begin <= identity { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }) {
            Ok(index) => index,
            Err(index) => index
        };
        if range_index == 0 || range_index > collection.len() {
            return false;
        }
        let is_not_last = range_index < collection.len();
        let mut range = &mut collection[range_index-1];
        if is_not_last && identity >= range.begin+range.length {
            return false;
        }
        if identity == range.begin {
            range.begin += 1;
            if is_not_last {
                range.length -= 1;
                if range.length == 0 {
                    collection.remove(range_index-1);
                }
            }
        } else if is_not_last && identity == range.begin+range.length-1 {
            range.length -= 1;
        } else {
            let count = identity-range.begin;
            let prev_begin = range.begin;
            range.begin = identity+1;
            if range.length > 0 {
                range.length -= 1+count;
            }
            collection.insert(range_index-1, IdentityRange{begin: prev_begin, length: count});
        }
        true
    }

    pub fn insert(&mut self, identity: Identity) -> bool {
        let collection = &mut self.0;
        let range_index = match collection.binary_search_by(|probe| if probe.begin <= identity { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }) {
            Ok(index) => index,
            Err(index) => index
        };
        let merge_prev_range = if range_index > 0 {
            match &collection.get(range_index-1) {
                Some(prev_range) => {
                    if range_index == collection.len() || identity < prev_range.begin+prev_range.length {
                        return false;
                    }
                    prev_range.begin+prev_range.length == identity
                },
                None => { false }
            }
        } else { false };
        let merge_next_range = match &collection.get(range_index) {
            Some(next_range) => { identity+1 == next_range.begin },
            None => { false }
        };
        if merge_prev_range && merge_next_range {
            let is_not_last = range_index+1 < collection.len();
            let prev_range = collection[range_index-1];
            let next_range = &mut collection[range_index];
            next_range.begin = prev_range.begin;
            if is_not_last {
                next_range.length += 1+prev_range.length;
            }
            collection.remove(range_index-1);
        } else if merge_prev_range {
            let prev_range = &mut collection[range_index-1];
            prev_range.length += 1;
        } else if merge_next_range {
            let next_range = &mut collection[range_index];
            next_range.begin -= 1;
            if next_range.length > 0 {
                next_range.length += 1;
            }
        } else {
            collection.insert(range_index, IdentityRange{begin: identity, length: 1});
        }
        true
    }
}

type GammaCollection = HashSet<Symbol>;
type BetaCollection = HashMap<Symbol, GammaCollection>;
type AlphaCollection = HashMap<Identity, SymbolHandle>;

struct SymbolHandle {
    data_content: RefCell<Box<[usize]>>,
    data_length: usize,
    subindices: [BetaCollection; 6]
}

struct NamespaceHandle {
    free_pool: IdentityPool,
    symbol_index: AlphaCollection
}

type NamespaceIndex = HashMap<Identity, NamespaceHandle>;

#[derive(Clone, Copy)]
pub enum TripleIndex {
    EAV, AVE, VEA,
    EVA, AEV, VAE
}

#[derive(Clone, Copy)]
enum TripleQueryFunc {
    SearchMMM,
    SearchMMI,
    SearchMII,
    SearchIII,
    SearchMMV,
    SearchMVV,
    SearchMVI,
    SearchVII,
    SearchVVI,
    SearchVVV
}

const INDEX_LOOKUP: [TripleIndex; 27] = [
    TripleIndex::EAV, TripleIndex::AVE, TripleIndex::AVE,
    TripleIndex::VEA, TripleIndex::VEA, TripleIndex::VAE,
    TripleIndex::VEA, TripleIndex::VEA, TripleIndex::VEA,
    TripleIndex::EAV, TripleIndex::AVE, TripleIndex::AVE,
    TripleIndex::EAV, TripleIndex::EAV, TripleIndex::AVE,
    TripleIndex::EVA, TripleIndex::VEA, TripleIndex::VEA,
    TripleIndex::EAV, TripleIndex::AEV, TripleIndex::AVE,
    TripleIndex::EAV, TripleIndex::EAV, TripleIndex::AVE,
    TripleIndex::EAV, TripleIndex::EAV, TripleIndex::EAV
];

const SEARCH_LOOKUP: [TripleQueryFunc; 27] = [
    TripleQueryFunc::SearchMMM, TripleQueryFunc::SearchMMV, TripleQueryFunc::SearchMMI,
    TripleQueryFunc::SearchMMV, TripleQueryFunc::SearchMVV, TripleQueryFunc::SearchMVI,
    TripleQueryFunc::SearchMMI, TripleQueryFunc::SearchMVI, TripleQueryFunc::SearchMII,
    TripleQueryFunc::SearchMMV, TripleQueryFunc::SearchMVV, TripleQueryFunc::SearchMVI,
    TripleQueryFunc::SearchMVV, TripleQueryFunc::SearchVVV, TripleQueryFunc::SearchVVI,
    TripleQueryFunc::SearchMVI, TripleQueryFunc::SearchVVI, TripleQueryFunc::SearchVII,
    TripleQueryFunc::SearchMMI, TripleQueryFunc::SearchMVI, TripleQueryFunc::SearchMII,
    TripleQueryFunc::SearchMVI, TripleQueryFunc::SearchVVI, TripleQueryFunc::SearchVII,
    TripleQueryFunc::SearchMII, TripleQueryFunc::SearchVII, TripleQueryFunc::SearchIII
];

type TriplePermutation = [[usize; 6]; 3];
const TRIPLE_PRIORITIZED: TriplePermutation = [
    [0, 1, 2, 0, 1, 2],
    [1, 2, 0, 2, 0, 1],
    [2, 0, 1, 1, 2, 0]
];
const TRIPLE_NORMALIZED: TriplePermutation = [
    [0, 2, 1, 0, 1, 2],
    [1, 0, 2, 2, 0, 1],
    [2, 1, 0, 1, 2, 0]
];

const META_NAMESPACE: usize = 2;

thread_local!(static NAMESPACE_INDEX: RefCell<NamespaceIndex> = RefCell::new(HashMap::new()));



fn manifest_namespace(namespace_index: &mut NamespaceIndex, namespace_identity: Identity) -> &mut NamespaceHandle {
    if namespace_index.contains_key(&namespace_identity) {
        return namespace_index.get_mut(&namespace_identity).unwrap();
    }
    let namespace_handle = NamespaceHandle{free_pool: IdentityPool::new(), symbol_index: AlphaCollection::new()};
    assert!(namespace_index.insert(namespace_identity, namespace_handle).is_none());
    get_symbol_handle_mut(namespace_index, Symbol{0: META_NAMESPACE, 1: namespace_identity}, true);
    return namespace_index.get_mut(&namespace_identity).unwrap();
}

fn unlink_namespace(namespace_identity: Identity) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        match namespace_index.get(&namespace_identity) {
            Some(namespace_handle) => {
                let mut triples: HashSet<Triple> = HashSet::new();
                let mut triple: Triple = [Symbol{0: 0, 1: 0}; 3];
                for (symbol_identity, symbol_handle) in namespace_handle.symbol_index.iter() {
                    triple[0] = Symbol{0: namespace_identity, 1: *symbol_identity};
                    for triple_index in [TripleIndex::EAV, TripleIndex::AVE, TripleIndex::VEA].into_iter() {
                        let beta_self = &symbol_handle.subindices[*triple_index as usize];
                        for (beta, gamma_self) in beta_self.iter() {
                            triple[1] = *beta;
                            if beta.0 != namespace_identity {
                                for gamma in gamma_self.iter() {
                                    triple[2] = *gamma;
                                    triples.insert(reorder_triple(&TRIPLE_NORMALIZED, *triple_index, &triple));
                                }
                            } else {
                                for gamma in gamma_self.iter() {
                                    if gamma.0 != namespace_identity {
                                        triple[2] = *gamma;
                                        triples.insert(reorder_triple(&TRIPLE_NORMALIZED, *triple_index, &triple));
                                    }
                                }
                            }
                        }
                    }
                }
                for triple in triples.iter() {
                    assert!(set_triple_internal(&mut namespace_index, *triple, false));
                }
                assert!(namespace_index.remove(&namespace_identity).is_some());
                true
            },
            None => { false }
        }
    })
}

fn get_symbol_handle_mut(namespace_index: &mut NamespaceIndex, symbol: Symbol, manifest: bool) -> Option<&mut SymbolHandle> {
    if manifest {
        let namespace_handle = manifest_namespace(namespace_index, symbol.0);
        if !namespace_handle.symbol_index.contains_key(&symbol.1) {
            let symbol_handle = SymbolHandle{data_content: RefCell::new(Box::new([])), data_length: 0, subindices: [BetaCollection::new(), BetaCollection::new(), BetaCollection::new(), BetaCollection::new(), BetaCollection::new(), BetaCollection::new()]};
            assert!(namespace_handle.symbol_index.insert(symbol.1, symbol_handle).is_none());
            assert!(namespace_handle.free_pool.remove(symbol.1));
        }
        return namespace_handle.symbol_index.get_mut(&symbol.1);
    } else {
        match namespace_index.get_mut(&symbol.0) {
            Some(namespace_handle) => namespace_handle.symbol_index.get_mut(&symbol.1),
            None => None
        }
    }
}

fn get_symbol_handle(namespace_index: &NamespaceIndex, symbol: Symbol) -> Option<&SymbolHandle> {
    match namespace_index.get(&symbol.0) {
        Some(namespace_handle) => namespace_handle.symbol_index.get(&symbol.1),
        None => None
    }
}

pub fn manifest_symbol(symbol: Symbol) {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        get_symbol_handle_mut(&mut namespace_index, symbol, true);
    });
}

pub fn create_symbol(namespace_identity: Identity) -> Symbol {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        let namespace_handle = manifest_namespace(&mut namespace_index, namespace_identity);
        let symbol_identity: Identity = namespace_handle.free_pool.get();
        let symbol = Symbol{0: namespace_identity, 1: symbol_identity};
        get_symbol_handle_mut(&mut namespace_index, symbol, true);
        symbol
    })
}

pub fn release_symbol(symbol: Symbol) -> bool {
    if NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        match namespace_index.get_mut(&symbol.0) {
            Some(namespace_handle) => {
                if namespace_handle.symbol_index.remove(&symbol.1).is_some() {
                    if namespace_handle.symbol_index.is_empty() {
                        assert!(namespace_index.remove(&symbol.0).is_some());
                    } else {
                        assert!(namespace_handle.free_pool.insert(symbol.1));
                    }
                } else {
                    return false;
                }
            },
            None => { return false; }
        };
        true
    }) {
        if symbol.0 == META_NAMESPACE {
            assert!(unlink_namespace(symbol.1));
        }
        true
    } else { false }
}



pub fn get_length(symbol: Symbol) -> usize {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let namespace_index = namespace_index_cell.borrow();
        match get_symbol_handle(&namespace_index, symbol) {
            Some(symbol_handle) => symbol_handle.data_length,
            None => 0
        }
    })
}

pub fn crease_length(symbol: Symbol, offset: usize, length: isize) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        match get_symbol_handle_mut(&mut namespace_index, symbol, false) {
            Some(symbol_handle) => {
                let length_abs: usize;
                if length < 0 {
                    length_abs = -length as usize;
                    if offset+length_abs > symbol_handle.data_length {
                        return false;
                    }
                } else {
                    length_abs = length as usize;
                    if offset > symbol_handle.data_length {
                        return false;
                    }
                }
                let new_data_length = ((symbol_handle.data_length as isize)+length) as usize;
                let mut new_data_content: Box<[usize]> = vec![0; (new_data_length+bitops::ARCHITECTURE_SIZE-1)/bitops::ARCHITECTURE_SIZE].into_boxed_slice();
                for i in 0..(offset+bitops::ARCHITECTURE_SIZE-1)/bitops::ARCHITECTURE_SIZE {
                    new_data_content[i] = symbol_handle.data_content.borrow()[i];
                }
                if offset%bitops::ARCHITECTURE_SIZE > 0 {
                    new_data_content[offset/bitops::ARCHITECTURE_SIZE] &= bitops::lsb_bitmask(offset%bitops::ARCHITECTURE_SIZE);
                }
                if length < 0 {
                    bitops::bitwise_copy_nonoverlapping(&mut new_data_content, &symbol_handle.data_content.borrow(), offset, offset+length_abs, symbol_handle.data_length-offset-length_abs);
                } else {
                    bitops::bitwise_copy_nonoverlapping(&mut new_data_content, &symbol_handle.data_content.borrow(), offset+length_abs, offset, symbol_handle.data_length-offset);
                }
                symbol_handle.data_length = new_data_length;
                symbol_handle.data_content.replace(new_data_content);
                return true;
            },
            None => false
        }
    })
}

pub fn read_data(symbol: Symbol, offset: usize, length: usize, dst: &mut [usize]) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let namespace_index = namespace_index_cell.borrow();
        match get_symbol_handle(&namespace_index, symbol) {
            Some(symbol_handle) => {
                if offset+length > symbol_handle.data_length {
                    return false;
                }
                let data_content = symbol_handle.data_content.borrow();
                let bitwise_read = bitops::BitwiseRead::new(&data_content, length, offset);
                let mut index: usize = 0;
                for src in bitwise_read {
                    dst[index] = src;
                    index += 1;
                }
                return true;
            },
            None => false
        }
    })
}

pub fn write_data(symbol: Symbol, offset: usize, length: usize, src: &[usize]) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        match get_symbol_handle_mut(&mut namespace_index, symbol, false) {
            Some(symbol_handle) => {
                if offset+length > symbol_handle.data_length {
                    return false;
                }
                let mut data_content = symbol_handle.data_content.borrow_mut();
                let mut bitwise_write = bitops::BitwiseWrite::new(&mut data_content, length, offset);
                let mut index: usize = 0;
                while bitwise_write.more() {
                    bitwise_write.next(src[index]);
                    index += 1;
                }
                return true;
            },
            None => false
        }
    })
}

pub fn replace_data(dst_symbol: Symbol, dst_offset: usize, src_symbol: Symbol, src_offset: usize, length: usize) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let namespace_index = namespace_index_cell.borrow();
        let dst_symbol_handle = match get_symbol_handle(&namespace_index, dst_symbol) {
            Some(symbol_handle) => symbol_handle,
            None => { return false; }
        };
        let src_symbol_handle = match get_symbol_handle(&namespace_index, src_symbol) {
            Some(symbol_handle) => symbol_handle,
            None => { return false; }
        };
        if dst_offset+length > dst_symbol_handle.data_length || src_offset+length > src_symbol_handle.data_length {
            return false;
        }
        bitops::bitwise_copy_nonoverlapping(&mut dst_symbol_handle.data_content.borrow_mut(), &src_symbol_handle.data_content.borrow(), dst_offset, src_offset, length);
        true
    })
}



fn set_triple_subindex(beta_self: &mut BetaCollection, beta: Symbol, gamma: Symbol, linked: bool) -> bool {
    if linked {
        match beta_self.get_mut(&beta) {
            Some(gamma_self) => { gamma_self.insert(gamma) },
            None => {
                let mut gamma_self = GammaCollection::new();
                assert!(gamma_self.insert(gamma));
                assert!(beta_self.insert(beta, gamma_self).is_none());
                true
            }
        }
    } else {
        match beta_self.get_mut(&beta) {
            Some(gamma_self) => {
                if !gamma_self.remove(&gamma) {
                    return false;
                }
                if gamma_self.is_empty() {
                    assert!(beta_self.remove(&beta).is_some());
                }
                true
            },
            None => { false }
        }
    }
}



fn set_triple_internal(namespace_index: &mut NamespaceIndex, triple: Triple, linked: bool) -> bool {
    let mut result: bool = false;
    for triple_index in 0..3 {
        match get_symbol_handle_mut(namespace_index, triple[triple_index], linked) {
            Some(entity_handle) => {
                if set_triple_subindex(&mut entity_handle.subindices[triple_index], triple[(triple_index+1)%3], triple[(triple_index+2)%3], linked) {
                    result = true;
                }
                if set_triple_subindex(&mut entity_handle.subindices[triple_index+3], triple[(triple_index+2)%3], triple[(triple_index+1)%3], linked) {
                    result = true;
                }
            },
            None => { }
        }
    }
    result
}

pub fn set_triple(triple: Triple, linked: bool) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        set_triple_internal(&mut namespace_index, triple, linked)
    })
}

pub fn query_symbols(namespace_identity: Identity) -> Vec<Identity> {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let namespace_index = namespace_index_cell.borrow();
        let mut result: Vec<Identity> = vec![];
        match namespace_index.get(&namespace_identity) {
            Some(namespace_handle) => {
                for key in namespace_handle.symbol_index.keys() {
                    result.push(*key);
                }
            },
            None => {}
        }
        result
    })
}

fn reorder_triple(order: &TriplePermutation, triple_index: TripleIndex, triple: &Triple) -> Triple {
    let index = triple_index as usize;
    return [triple[order[0][index]], triple[order[1][index]], triple[order[2][index]]];
}

pub fn query_triples(mask: usize, mut triple: Triple) -> Vec<Triple> {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let namespace_index = namespace_index_cell.borrow();
        let triple_index = INDEX_LOOKUP[mask];
        triple = reorder_triple(&TRIPLE_PRIORITIZED, triple_index, &triple);
        let mut result: Vec<Triple> = vec![];
        match SEARCH_LOOKUP[mask] {
            TripleQueryFunc::SearchMMM => {
                match get_symbol_handle(&namespace_index, triple[0]) {
                    Some(symbol_handle) => {
                        match symbol_handle.subindices[triple_index as usize].get(&triple[1]) {
                            Some(gamma_self) => {
                                if gamma_self.contains(&triple[2]) {
                                    result.push(triple);
                                }
                            },
                            None => {}
                        }
                    },
                    None => {}
                }
            },
            TripleQueryFunc::SearchMMI => {
                match get_symbol_handle(&namespace_index, triple[0]) {
                    Some(symbol_handle) => {
                        if symbol_handle.subindices[triple_index as usize].contains_key(&triple[1]) {
                            result.push(reorder_triple(&TRIPLE_NORMALIZED, triple_index, &triple));
                        }
                    },
                    None => {}
                }
            },
            TripleQueryFunc::SearchMII => {
                match get_symbol_handle(&namespace_index, triple[0]) {
                    Some(symbol_handle) => {
                        if !symbol_handle.subindices[triple_index as usize].is_empty() {
                            result.push(reorder_triple(&TRIPLE_NORMALIZED, triple_index, &triple));
                        }
                    },
                    None => {}
                }
            },
            TripleQueryFunc::SearchIII => {
                for namespace_handle in namespace_index.values() {
                    for symbol_handle in namespace_handle.symbol_index.values() {
                        if !symbol_handle.subindices[triple_index as usize].is_empty() {
                            result.push(triple);
                            break;
                        }
                    }
                }
            },
            TripleQueryFunc::SearchMMV => {
                match get_symbol_handle(&namespace_index, triple[0]) {
                    Some(symbol_handle) => {
                        match symbol_handle.subindices[triple_index as usize].get(&triple[1]) {
                            Some(gamma_self) => {
                                for gamma in gamma_self.iter() {
                                    triple[2] = *gamma;
                                    result.push(reorder_triple(&TRIPLE_NORMALIZED, triple_index, &triple));
                                }
                            },
                            None => {}
                        }
                    },
                    None => {}
                }
            },
            TripleQueryFunc::SearchMVV => {
                match get_symbol_handle(&namespace_index, triple[0]) {
                    Some(symbol_handle) => {
                        for (beta, gamma_self) in symbol_handle.subindices[triple_index as usize].iter() {
                            triple[1] = *beta;
                            for gamma in gamma_self.iter() {
                                triple[2] = *gamma;
                                result.push(reorder_triple(&TRIPLE_NORMALIZED, triple_index, &triple));
                            }
                        }
                    },
                    None => {}
                }
            },
            TripleQueryFunc::SearchMVI => {
                match get_symbol_handle(&namespace_index, triple[0]) {
                    Some(symbol_handle) => {
                        for beta in symbol_handle.subindices[triple_index as usize].keys() {
                            triple[1] = *beta;
                            result.push(reorder_triple(&TRIPLE_NORMALIZED, triple_index, &triple));
                        }
                    },
                    None => {}
                }
            },
            TripleQueryFunc::SearchVII => {
                for (namespace_identity, namespace_handle) in namespace_index.iter() {
                    for (symbol_identity, symbol_handle) in namespace_handle.symbol_index.iter() {
                        if symbol_handle.subindices[triple_index as usize].is_empty() {
                            continue;
                        }
                        triple[0] = Symbol{0: *namespace_identity, 1: *symbol_identity};
                        result.push(reorder_triple(&TRIPLE_NORMALIZED, triple_index, &triple));
                    }
                }
            },
            TripleQueryFunc::SearchVVI => {
                for (namespace_identity, namespace_handle) in namespace_index.iter() {
                    for (symbol_identity, symbol_handle) in namespace_handle.symbol_index.iter() {
                        triple[0] = Symbol{0: *namespace_identity, 1: *symbol_identity};
                        for beta in symbol_handle.subindices[triple_index as usize].keys() {
                            triple[1] = *beta;
                            result.push(reorder_triple(&TRIPLE_NORMALIZED, triple_index, &triple));
                        }
                    }
                }
            },
            TripleQueryFunc::SearchVVV => {
                for (namespace_identity, namespace_handle) in namespace_index.iter() {
                    for (symbol_identity, symbol_handle) in namespace_handle.symbol_index.iter() {
                        triple[0] = Symbol{0: *namespace_identity, 1: *symbol_identity};
                        for (beta, gamma_self) in symbol_handle.subindices[triple_index as usize].iter() {
                            triple[1] = *beta;
                            for gamma in gamma_self.iter() {
                                triple[2] = *gamma;
                                result.push(triple);
                            }
                        }
                    }
                }
            }
        }
        result
    })
}
