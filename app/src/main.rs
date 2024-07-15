use ts_block_builder;

ts_block! {{
    ##### test block #####
}}

fn main() {
    let name_ptr = name.as_ptr();
    let r = ts_block!( {##### test block2 #####} );
    assert_eq!(r, 42)
}
