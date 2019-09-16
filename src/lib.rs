#![allow(non_snake_case)]
mod bitops;
mod graph;
use wasm_bindgen::prelude::*;

unsafe fn transmute_vec<S, T>(mut vec: Vec<S>) -> Vec<T> {
    let ptr = vec.as_mut_ptr() as *mut T;
    let len = vec.len()*std::mem::size_of::<S>()/std::mem::size_of::<T>();
    let capacity = vec.capacity()*std::mem::size_of::<S>()/std::mem::size_of::<T>();
    std::mem::forget(vec);
    Vec::from_raw_parts(ptr, len, capacity)
}

#[wasm_bindgen]
pub fn manifestSymbol(namespace_identity: graph::Identity, symbol_identity: graph::Identity) {
    graph::manifest_symbol(graph::Symbol{0: namespace_identity, 1: symbol_identity})
}

#[wasm_bindgen]
pub fn createSymbol(namespace_identity: graph::Identity) -> graph::Identity {
    graph::create_symbol(namespace_identity).1
}

#[wasm_bindgen]
pub fn releaseSymbol(namespace_identity: graph::Identity, symbol_identity: graph::Identity) -> bool {
    graph::release_symbol(graph::Symbol{0: namespace_identity, 1: symbol_identity})
}

#[wasm_bindgen]
pub fn setTriple(entity_namespace_identity: graph::Identity, entity_symbol_identity: graph::Identity,
                 attribute_namespace_identity: graph::Identity, attribute_symbol_identity: graph::Identity,
                 value_namespace_identity: graph::Identity, value_symbol_identity: graph::Identity, linked: bool) -> bool {
    graph::set_triple([
        graph::Symbol{0: entity_namespace_identity, 1: entity_symbol_identity},
        graph::Symbol{0: attribute_namespace_identity, 1: attribute_symbol_identity},
        graph::Symbol{0: value_namespace_identity, 1: value_symbol_identity}
    ], linked)
}

#[wasm_bindgen]
pub fn querySymbols(namespaceIdentity: graph::Identity) -> Vec<graph::Identity> {
    graph::query_symbols(namespaceIdentity)
}

#[wasm_bindgen]
pub fn queryTriples(mask: usize,
                    entity_namespace_identity: graph::Identity, entity_symbol_identity: graph::Identity,
                    attribute_namespace_identity: graph::Identity, attribute_symbol_identity: graph::Identity,
                    value_namespace_identity: graph::Identity, value_symbol_identity: graph::Identity) -> Vec<graph::Identity> {
    let result = graph::query_triples(mask, [
        graph::Symbol{0: entity_namespace_identity, 1: entity_symbol_identity},
        graph::Symbol{0: attribute_namespace_identity, 1: attribute_symbol_identity},
        graph::Symbol{0: value_namespace_identity, 1: value_symbol_identity}
    ]);
    unsafe { transmute_vec::<graph::Triple, graph::Identity>(result) }
}

#[wasm_bindgen]
pub fn getLength(namespace_identity: graph::Identity, symbol_identity: graph::Identity) -> usize {
    graph::get_length(graph::Symbol{0: namespace_identity, 1: symbol_identity})
}

#[wasm_bindgen]
pub fn creaseLength(namespace_identity: graph::Identity, symbol_identity: graph::Identity, offset: usize, length: isize) -> bool {
    graph::crease_length(graph::Symbol{0: namespace_identity, 1: symbol_identity}, offset, length)
}

#[wasm_bindgen]
pub fn readData(namespace_identity: graph::Identity, symbol_identity: graph::Identity, offset: usize, length: usize, dst: &mut [usize]) -> bool {
    graph::read_data(graph::Symbol{0: namespace_identity, 1: symbol_identity}, offset, length, dst)
}

#[wasm_bindgen]
pub fn writeData(namespace_identity: graph::Identity, symbol_identity: graph::Identity, offset: usize, length: usize, src: &[usize]) -> bool {
    graph::write_data(graph::Symbol{0: namespace_identity, 1: symbol_identity}, offset, length, src)
}

#[wasm_bindgen]
pub fn replaceData(dst_namespace_identity: graph::Identity, dst_symbol_identity: graph::Identity, dst_offset: usize,
                   src_namespace_identity: graph::Identity, src_symbol_identity: graph::Identity, src_offset: usize,
                   length: usize) -> bool {
    graph::replace_data(
        graph::Symbol{0: dst_namespace_identity, 1: dst_symbol_identity}, dst_offset,
        graph::Symbol{0: src_namespace_identity, 1: src_symbol_identity}, src_offset,
        length
    )
}
