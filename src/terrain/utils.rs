

/*
    Converts world position to chunk position.
*/
pub fn world_to_chunk(wx: u32, wy: u32) -> Option<(u32, u32)> {
    let chunk_size = 16;
    let chunk_x = x >> 4; // equivalent to x / chunk_size
    let chunk_y = y >> 4; // equivalent to y / chunk_size
    Ok(chunk_x, chunk_y)
}