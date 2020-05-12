use std::collections::HashMap;
use std::collections::HashSet;
use std::cell::RefCell;
use crate::bitops;
use crate::symbol;

pub type Triple = [symbol::Symbol; 3];

type GammaCollection = HashSet<symbol::Symbol>;
type BetaCollection = HashMap<symbol::Symbol, GammaCollection>;
type AlphaCollection = HashMap<symbol::Identity, SymbolHandle>;

struct SymbolHandle {
    data_content: RefCell<Box<[usize]>>,
    data_length: usize,
    subindices: [BetaCollection; 6]
}

struct NamespaceHandle {
    free_pool: symbol::IdentityPool,
    symbol_index: AlphaCollection
}

type NamespaceIndex = HashMap<symbol::Identity, NamespaceHandle>;

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

const META_NAMESPACE_IDENTITY: usize = 0;

thread_local!(static NAMESPACE_INDEX: RefCell<NamespaceIndex> = RefCell::new(HashMap::new()));



fn manifest_namespace(namespace_index: &mut NamespaceIndex, namespace_identity: symbol::Identity) {
    if !namespace_index.contains_key(&namespace_identity) {
        let namespace_handle = NamespaceHandle{free_pool: symbol::IdentityPool::new(), symbol_index: AlphaCollection::new()};
        assert!(namespace_index.insert(namespace_identity, namespace_handle).is_none());
    }
}

fn get_symbol_handle_mut(namespace_index: &mut NamespaceIndex, symbol: symbol::Symbol) -> Option<&mut SymbolHandle> {
    match namespace_index.get_mut(&symbol.0) {
        Some(namespace_handle) => namespace_handle.symbol_index.get_mut(&symbol.1),
        None => None
    }
}

fn get_symbol_handle(namespace_index: &NamespaceIndex, symbol: symbol::Symbol) -> Option<&SymbolHandle> {
    match namespace_index.get(&symbol.0) {
        Some(namespace_handle) => namespace_handle.symbol_index.get(&symbol.1),
        None => None
    }
}

fn manifest_symbol_internal(namespace_index: &mut NamespaceIndex, symbol: symbol::Symbol) -> bool {
    if symbol == symbol::Symbol(META_NAMESPACE_IDENTITY, META_NAMESPACE_IDENTITY) {
        manifest_namespace(namespace_index, META_NAMESPACE_IDENTITY);
    }
    let namespace_handle = namespace_index.get_mut(&symbol.0).unwrap();
    if namespace_handle.symbol_index.contains_key(&symbol.1) {
        return false;
    }
    let symbol_handle = SymbolHandle{data_content: RefCell::new(Box::new([])), data_length: 0, subindices: [BetaCollection::new(), BetaCollection::new(), BetaCollection::new(), BetaCollection::new(), BetaCollection::new(), BetaCollection::new()]};
    assert!(namespace_handle.symbol_index.insert(symbol.1, symbol_handle).is_none());
    assert!(namespace_handle.free_pool.remove(symbol.1));
    if symbol.0 == META_NAMESPACE_IDENTITY {
        manifest_namespace(namespace_index, symbol.1);
    }
    return true;
}

pub fn manifest_symbol(symbol: symbol::Symbol) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        manifest_symbol_internal(&mut namespace_index, symbol)
    })
}

pub fn create_symbol(namespace_identity: symbol::Identity) -> symbol::Symbol {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        let namespace_handle = namespace_index.get_mut(&namespace_identity).unwrap();
        let symbol_identity: symbol::Identity = namespace_handle.free_pool.get();
        let symbol = symbol::Symbol{0: namespace_identity, 1: symbol_identity};
        manifest_symbol_internal(&mut namespace_index, symbol);
        symbol
    })
}

pub fn release_symbol(symbol: symbol::Symbol) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        match namespace_index.get_mut(&symbol.0) {
            Some(namespace_handle) => {
                match namespace_handle.symbol_index.get(&symbol.1) {
                    Some(symbol_handle) => {
                        assert!(symbol_handle.data_length == 0);
                        for subindex in &symbol_handle.subindices {
                            assert!(subindex.len() == 0);
                        }
                    },
                    None => { return false; }
                }
                assert!(namespace_handle.symbol_index.remove(&symbol.1).is_some());
                assert!(namespace_handle.free_pool.insert(symbol.1));
            },
            None => { return false; }
        };
        if symbol.0 == META_NAMESPACE_IDENTITY {
            assert!(namespace_index.remove(&symbol.1).is_some());
        }
        true
    })
}



pub fn get_length(symbol: symbol::Symbol) -> usize {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let namespace_index = namespace_index_cell.borrow();
        match get_symbol_handle(&namespace_index, symbol) {
            Some(symbol_handle) => symbol_handle.data_length,
            None => 0
        }
    })
}

pub fn crease_length(symbol: symbol::Symbol, offset: usize, length: isize) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        match get_symbol_handle_mut(&mut namespace_index, symbol) {
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
                true
            },
            None => false
        }
    })
}

pub fn read_data(symbol: symbol::Symbol, offset: usize, length: usize, dst: &mut [usize]) -> bool {
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
                true
            },
            None => false
        }
    })
}

pub fn write_data(symbol: symbol::Symbol, offset: usize, length: usize, src: &[usize]) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        match get_symbol_handle_mut(&mut namespace_index, symbol) {
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
                true
            },
            None => false
        }
    })
}

pub fn replace_data(dst_symbol: symbol::Symbol, dst_offset: usize, src_symbol: symbol::Symbol, src_offset: usize, length: usize) -> bool {
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



fn set_triple_subindex(beta_self: &mut BetaCollection, beta: symbol::Symbol, gamma: symbol::Symbol, linked: bool) -> bool {
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
            None => false
        }
    }
}



fn set_triple_internal(namespace_index: &mut NamespaceIndex, triple: Triple, linked: bool) -> bool {
    for triple_index in 0..3 {
        if get_symbol_handle_mut(namespace_index, triple[triple_index]).is_none() {
            return false;
        }
    }
    let mut result: bool = false;
    for triple_index in 0..3 {
        let entity_handle = get_symbol_handle_mut(namespace_index, triple[triple_index]).unwrap();
        result |= set_triple_subindex(&mut entity_handle.subindices[triple_index], triple[(triple_index+1)%3], triple[(triple_index+2)%3], linked);
        result |= set_triple_subindex(&mut entity_handle.subindices[triple_index+3], triple[(triple_index+2)%3], triple[(triple_index+1)%3], linked);
    }
    result
}

pub fn set_triple(triple: Triple, linked: bool) -> bool {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let mut namespace_index = namespace_index_cell.borrow_mut();
        set_triple_internal(&mut namespace_index, triple, linked)
    })
}

pub fn query_symbols(namespace_identity: symbol::Identity) -> Vec<symbol::Identity> {
    NAMESPACE_INDEX.with(|namespace_index_cell| {
        let namespace_index = namespace_index_cell.borrow();
        let mut result: Vec<symbol::Identity> = vec![];
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
                        triple[0] = symbol::Symbol{0: *namespace_identity, 1: *symbol_identity};
                        result.push(reorder_triple(&TRIPLE_NORMALIZED, triple_index, &triple));
                    }
                }
            },
            TripleQueryFunc::SearchVVI => {
                for (namespace_identity, namespace_handle) in namespace_index.iter() {
                    for (symbol_identity, symbol_handle) in namespace_handle.symbol_index.iter() {
                        triple[0] = symbol::Symbol{0: *namespace_identity, 1: *symbol_identity};
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
                        triple[0] = symbol::Symbol{0: *namespace_identity, 1: *symbol_identity};
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
