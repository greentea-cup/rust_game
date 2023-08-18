#![allow(dead_code)]

#[derive(Debug)]
pub struct Atlas {
    pub texture: Vec<u8>,
    pub size: usize,
    pub map: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TexurePosition {
    Dot(usize),
    Span(usize, usize),
}

const fn level_to_offset(mut level: u32) -> usize {
    // const MAX_OFFSET: usize = 0b01010101_01010101_01010101_01010101_01010101_01010101_01010101_01010101;
    // if level == 0 { 0 } else { MAX_OFFSET & ((1 << (2 * level - 1)) - 1) }
    let mut offset = 0;
    while level > 0 {
        offset <<= 2;
        offset |= 1;
        level -= 1;
    }
    offset
}

const fn full_level_space(level: u32) -> usize {
    level_to_offset(level + 1)
}

const fn one_level_space(level: u32) -> usize {
    4usize.pow(level)
}

const fn position_to_lo(position: usize) -> (u32, usize) {
    let mut level = 0;
    let mut offset = 0;
    while position >= offset {
        offset <<= 2;
        offset |= 1;
        level += 1;
    }
    (level - 1, offset >> 2)
}

const fn position_to_level(position: usize) -> u32 {
    position_to_lo(position).0
}

const fn position_to_offset(position: usize) -> usize {
    position_to_lo(position).1
}

const fn relative_position(position: usize) -> usize {
    let offset = position_to_offset(position);
    position - offset
}

fn texture_level_unchecked(size: usize, atlas_size: usize) -> u32 {
    atlas_size.ilog2() - size.ilog2()
}

fn texture_level(size: usize, atlas_size: usize) -> Option<u32> {
    if size > atlas_size || !(size.is_power_of_two() && atlas_size.is_power_of_two()) {
        return None;
    }
    Some(texture_level_unchecked(size, atlas_size))
}

fn upper_level_position(position: usize) -> usize {
    let rel = relative_position(position);
    let level = position_to_level(position);
    let brel = rel / 4;
    if level == 0 {
        return brel;
    }
    let boffset = level_to_offset(level.wrapping_sub(1));
    brel + boffset
}

const fn lower_level_span(position_or_span: TexurePosition) -> TexurePosition {
    match position_or_span {
        TexurePosition::Dot(position) => {
            let level = position_to_level(position);
            let rel = relative_position(position);
            let nrel0 = rel * 4;
            let nrel1 = nrel0 + 4;
            let noffset = level_to_offset(level + 1);
            TexurePosition::Span(nrel0 + noffset, nrel1 + noffset)
        },
        TexurePosition::Span(a, b) => {
            let level = position_to_level(a);
            let rel0 = relative_position(a);
            let rel1 = relative_position(b - 1);
            let nrel0 = rel0 * 4;
            let nrel1 = rel1 * 4 + 4;
            let noffset = level_to_offset(level + 1);
            TexurePosition::Span(nrel0 + noffset, nrel1 + noffset)
        },
    }
}

fn leveled_position(mut position: usize, max_level: u32) -> (Vec<TexurePosition>, u32) {
    let mut res = Vec::new();
    let level = position_to_level(position);
    let orig_position = position;
    for _ in 0..1 + level as usize {
        res.push(TexurePosition::Dot(position));
        position = upper_level_position(position);
    }
    res.reverse();
    let mut position = TexurePosition::Dot(orig_position);
    for _ in 1 + level as usize..1 + max_level as usize {
        let lpos = lower_level_span(position);
        position = lpos;
        res.push(lpos);
    }
    (res, level)
}

const fn level_scale(level: u32) -> usize {
    2usize.pow(level)
}

fn first_free_position(level: u32, occupied: &[bool], max_level: u32) -> Option<usize> {
    let mut offset = level_to_offset(level);
    let total_space = full_level_space(max_level);
    while offset > total_space {
        // not enough levels (texture too small)
        // treat texture bigger than in is
        // or return None
        offset >>= 2;
        // TODO: find better way to handle relatively small textures
    }
    let space = one_level_space(level);
    occupied
        .iter()
        .enumerate()
        .skip(offset)
        .take(space)
        .find(|x| !*x.1)
        .map(|x| x.0)
}

fn occupy(position: usize, occupied: &mut [bool], max_level: u32) {
    let pos = leveled_position(position, max_level).0;
    for p in pos {
        match p {
            TexurePosition::Dot(a) => {
                occupied[a] = true;
            },
            TexurePosition::Span(a, b) => {
                occupied.iter_mut().take(b).skip(a).for_each(|x| *x = true);
            },
        }
    }
}

fn position_to_block(position: usize) -> (usize, usize) {
    relative_position_to_block(relative_position(position))
}

fn relative_position_to_block(mut rel: usize) -> (usize, usize) {
    const POS: [(usize, usize); 4] = [(0, 0), (1, 0), (0, 1), (1, 1)];
    let mut k = 1;
    let (mut x, mut y) = (0, 0);
    while rel > 0 {
        let x1 = rel & 1;
        let y1 = (rel >> 1) & 1;
        x += k * x1;
        y += k * y1;
        rel /= 4;
        k *= 2;
    }
    (x, y)
}

fn block_to_tx(block: (usize, usize), level: u32, atlas_size: usize) -> (usize, usize) {
    let scale = level_scale(level);
    let block_size = atlas_size / scale;
    let (x, y) = block;
    (x * block_size, y * block_size)
}

fn place_texture(
    atlas: &mut [u8],
    atlas_size: usize,
    texture: &[u8],
    texture_size: usize,
    texture_position: (usize, usize),
    channels: usize,
) {
    let (x, y) = texture_position;
    let (nx, ny) = (x, atlas_size - y - texture_size);
    for (i, row) in (ny..ny + texture_size).enumerate() {
        for (j, col) in (nx..nx + texture_size).enumerate() {
            for k in 0..channels {
                atlas[channels * row * atlas_size + channels * col + k] =
                    texture[channels * i * texture_size + channels * j + k];
            }
        }
    }
}

pub fn textures_to_atlas(
    textures: &[image::DynamicImage],
    atlas_size: usize,
    alpha: bool,
    max_level: u32,
) -> (Atlas, Option<Vec<usize>>) {
    let mut map = vec![0; textures.len()];
    let channels = if alpha { 4 } else { 3 };
    let total = channels * atlas_size * atlas_size;
    let mut atlas = vec![0; total];
    let mut skipped = Vec::new();
    let mut occupied = vec![false; full_level_space(max_level)];
    for (i, tx) in textures.iter().enumerate() {
        // assume all textures are squares and are size of power of two
        let size = tx.width() as usize;
        if size > atlas_size {
            skipped.push(i);
            continue;
        }
        let level = texture_level_unchecked(size, atlas_size);
        let pos = first_free_position(level, &occupied, max_level);
        if let Some(pos) = pos {
            occupy(pos, &mut occupied, max_level);
            map[i] = pos;
            let block = position_to_block(pos);
            let tx_pos = block_to_tx(block, level, atlas_size);
            let s = if alpha {
                tx.clone().into_rgba8().into_flat_samples()
            } else {
                tx.clone().into_rgb8().into_flat_samples()
            };
            place_texture(&mut atlas, atlas_size, &s.samples, size, tx_pos, channels);
        } else {
            skipped.push(i);
        }
    }
    (
        Atlas {
            texture: atlas,
            size: atlas_size,
            map,
        },
        if skipped.is_empty() {
            None
        } else {
            Some(skipped)
        },
    )
}

pub fn adjust_uvs(uvs: &[f32], position: usize) -> Vec<f32> {
    use crate::memcast;
    let block = position_to_block(position);
    let level = position_to_level(position);
    let scale = 1.0 / level_scale(level) as f32;
    let fblock = (block.0 as f32 * scale, block.1 as f32 * scale);
    let uvs1 = memcast::slice_cast::<f32, [f32; 2]>(uvs, uvs.len() / 2);
    let a = uvs1
        .iter()
        .flat_map(|[x, y]|
            // u, 1 - v
            [x * scale + fblock.0, 1. - (y * scale + fblock.1)])
        .collect::<_>();
    a
}
