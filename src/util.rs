pub fn get_default_offset(offset_opt: &Option<i64>) -> &i64 {
    match offset_opt {
        Some(ref offset) => offset,
        None => &0,
    }
}
