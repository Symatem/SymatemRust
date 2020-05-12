#![allow(non_snake_case)]
use std::cell::RefCell;
mod bitops;
mod symbol;
mod graph;
use wasm_bindgen::prelude::*;

unsafe fn transmute_vec<S, T>(mut vec: Vec<S>) -> Vec<T> {
    let ptr = vec.as_mut_ptr() as *mut T;
    let len = vec.len()*std::mem::size_of::<S>()/std::mem::size_of::<T>();
    let capacity = vec.capacity()*std::mem::size_of::<S>()/std::mem::size_of::<T>();
    std::mem::forget(vec);
    Vec::from_raw_parts(ptr, len, capacity)
}



thread_local!(static IDENTITY_POOL: RefCell<symbol::IdentityPool> = RefCell::new(symbol::IdentityPool::new()));

#[wasm_bindgen]
pub fn testIdentityPoolRanges() -> Vec<usize> {
    IDENTITY_POOL.with(|identity_pool_cell| {
        let mut identity_pool = identity_pool_cell.borrow_mut();
        let result = identity_pool.get_ranges();
        unsafe { transmute_vec::<symbol::IdentityRange, usize>(result.to_vec()) }
    })
}

#[wasm_bindgen]
pub fn testIdentityPoolRemove(identity: symbol::Identity) -> bool {
    IDENTITY_POOL.with(|identity_pool_cell| {
        let mut identity_pool = identity_pool_cell.borrow_mut();
        identity_pool.remove(identity)
    })
}

#[wasm_bindgen]
pub fn testIdentityPoolInsert(identity: symbol::Identity) -> bool {
    IDENTITY_POOL.with(|identity_pool_cell| {
        let mut identity_pool = identity_pool_cell.borrow_mut();
        identity_pool.insert(identity)
    })
}



#[wasm_bindgen]
pub fn manifestSymbol(namespace_identity: symbol::Identity, symbol_identity: symbol::Identity) -> bool {
    graph::manifest_symbol(symbol::Symbol(namespace_identity, symbol_identity))
}

#[wasm_bindgen]
pub fn createSymbol(namespace_identity: symbol::Identity) -> symbol::Identity {
    graph::create_symbol(namespace_identity).1
}

#[wasm_bindgen]
pub fn releaseSymbol(namespace_identity: symbol::Identity, symbol_identity: symbol::Identity) -> bool {
    graph::release_symbol(symbol::Symbol(namespace_identity, symbol_identity))
}

#[wasm_bindgen]
pub fn setTriple(entity_namespace_identity: symbol::Identity, entity_symbol_identity: symbol::Identity,
                 attribute_namespace_identity: symbol::Identity, attribute_symbol_identity: symbol::Identity,
                 value_namespace_identity: symbol::Identity, value_symbol_identity: symbol::Identity, linked: bool) -> bool {
    graph::set_triple([
        symbol::Symbol(entity_namespace_identity, entity_symbol_identity),
        symbol::Symbol(attribute_namespace_identity, attribute_symbol_identity),
        symbol::Symbol(value_namespace_identity, value_symbol_identity)
    ], linked)
}

#[wasm_bindgen]
pub fn querySymbols(namespaceIdentity: symbol::Identity) -> Vec<symbol::Identity> {
    graph::query_symbols(namespaceIdentity)
}

#[wasm_bindgen]
pub fn queryTriples(mask: usize,
                    entity_namespace_identity: symbol::Identity, entity_symbol_identity: symbol::Identity,
                    attribute_namespace_identity: symbol::Identity, attribute_symbol_identity: symbol::Identity,
                    value_namespace_identity: symbol::Identity, value_symbol_identity: symbol::Identity) -> Vec<symbol::Identity> {
    let result = graph::query_triples(mask, [
        symbol::Symbol(entity_namespace_identity, entity_symbol_identity),
        symbol::Symbol(attribute_namespace_identity, attribute_symbol_identity),
        symbol::Symbol(value_namespace_identity, value_symbol_identity)
    ]);
    unsafe { transmute_vec::<graph::Triple, symbol::Identity>(result) }
}

#[wasm_bindgen]
pub fn getLength(namespace_identity: symbol::Identity, symbol_identity: symbol::Identity) -> usize {
    graph::get_length(symbol::Symbol(namespace_identity, symbol_identity))
}

#[wasm_bindgen]
pub fn creaseLength(namespace_identity: symbol::Identity, symbol_identity: symbol::Identity, offset: usize, length: isize) -> bool {
    graph::crease_length(symbol::Symbol(namespace_identity, symbol_identity), offset, length)
}

#[wasm_bindgen]
pub fn readData(namespace_identity: symbol::Identity, symbol_identity: symbol::Identity, offset: usize, length: usize, dst: &mut [usize]) -> bool {
    graph::read_data(symbol::Symbol(namespace_identity, symbol_identity), offset, length, dst)
}

#[wasm_bindgen]
pub fn writeData(namespace_identity: symbol::Identity, symbol_identity: symbol::Identity, offset: usize, length: usize, src: &[usize]) -> bool {
    graph::write_data(symbol::Symbol(namespace_identity, symbol_identity), offset, length, src)
}

#[wasm_bindgen]
pub fn replaceData(dst_namespace_identity: symbol::Identity, dst_symbol_identity: symbol::Identity, dst_offset: usize,
                   src_namespace_identity: symbol::Identity, src_symbol_identity: symbol::Identity, src_offset: usize,
                   length: usize) -> bool {
    graph::replace_data(
        symbol::Symbol(dst_namespace_identity, dst_symbol_identity), dst_offset,
        symbol::Symbol(src_namespace_identity, src_symbol_identity), src_offset,
        length
    )
}
