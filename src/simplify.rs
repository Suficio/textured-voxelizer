use crate::octree::{ VoxelTree, TreeBody };
use crate::color::*;

use cgmath::{ Vector3, Vector4 };

pub fn simplify(octree: &mut VoxelTree::<Vector4::<u8>>, write_data: &mut brs::WriteData) {
    let colorset = convert_colorset_to_hsv(&write_data.colors);

    loop {
        let mut colors = Vec::<Vector4::<u8>>::new();
        let x; let y; let z;
        {
            let (location, voxel) = octree.get_any_mut_or_create();

            x = location[0];
            y = location[1];
            z = location[2];

            match voxel {
                TreeBody::Leaf(c) => {
                    colors.push(*c);
                },
                _ => { break }
            }
        }

        let mut xp = x + 1;
        let mut yp = y + 1;
        let mut zp = z + 1;

        // Expand z direction first due to octree ordering followed by y
        // Ensures blocks are simplified in the pattern of Morton coding
        while zp - z < 200 {
            let voxel = octree.get_mut_or_create(Vector3::new(x, y, zp), 0);
            match voxel {
                TreeBody::Leaf(c) => {
                    colors.push(*c);
                    zp += 1
                },
                _ => { break }
            }
        }

        while yp - y < 200 {
            let mut pass = true;
            for sz in z..zp {
                let voxel = octree.get_mut_or_create(Vector3::new(x, yp, sz), 0);
                match voxel {
                    TreeBody::Leaf(c) => colors.push(*c),
                    _ => { pass = false; break }
                }
            }
            if !pass { break }
            yp += 1;
        }

        while xp - x < 200 {
            let mut pass = true;
            for sy in y..yp {
                for sz in z..zp {
                    let voxel = octree.get_mut_or_create(Vector3::new(xp, sy, sz), 0);
                    match voxel {
                        TreeBody::Leaf(c) => colors.push(*c),
                        _ => { pass = false; break }
                    }
                }
                if !pass { break }
            }
            if !pass { break }
            xp += 1;
        }

        // Clear nodes
        // This cant be done during the loops above unless you keep track
        // of which nodes you have already deleted
        for sx in x..xp {
            for sy in y..yp {
                for sz in z..zp {
                    let voxel = octree.get_mut_or_create(Vector3::new(sx, sy, sz), 0);
                    *voxel = TreeBody::Empty;
                }
            }
        }

        let color = match_hsv_to_colorset(&colorset, &hsv_average(&colors));

        let w = xp - x;
        let h = yp - y;
        let d = zp - z;

        write_data.bricks.push(
            brs::Brick {
                asset_name_index: 0,
                // Coordinates are rotated
                size: (5*w as u32, 5*d as u32, 2*h as u32),
                position: (
                    (5*w + 10*x) as i32,
                    (5*d + 10*z) as i32,
                    (2*h + 4*y) as i32
                ),
                direction: brs::Direction::ZPositive,
                rotation: brs::Rotation::Deg0,
                collision: true,
                visibility: true,
                material_index: 2,
                color: brs::ColorMode::Set(color as u32),
                owner_index: None
            }
        );
    }
}

pub fn simplify_lossless(octree: &mut VoxelTree::<Vector4::<u8>>, write_data: &mut brs::WriteData) {
    let d: isize = 1 << octree.size;
    let len = d + 1;

    let colorset = convert_colorset_to_hsv(&write_data.colors);

    loop {
        let color;
        let x; let y; let z;
        {
            let (location, voxel) = octree.get_any_mut_or_create();
            
            x = location[0];
            y = location[1];
            z = location[2];

            match voxel {
                TreeBody::Leaf(c) => {
                    color = match_hsv_to_colorset(&colorset, &rgb2hsv(*c));
                },
                _ => { break }
            }
        }

        let mut xp = x + 1;
        let mut yp = y + 1;
        let mut zp = z + 1;

        // Expand z direction first due to octree ordering followed by y
        // Ensures blocks are simplified in the pattern of Morton coding
        while zp < len && (zp - z) < 200 {
            let voxel = octree.get_mut_or_create(Vector3::new(x, y, zp), 0);
            match voxel {
                TreeBody::Leaf(c) => {
                    let color_temp = match_hsv_to_colorset(&colorset, &rgb2hsv(*c));
                    if color_temp != color { break }
                    zp += 1;
                },
                _ => { break }
            }
        }

        while yp < len && (yp - y) < 200 {
            let mut pass = true;
            for sz in z..zp {
                let voxel = octree.get_mut_or_create(Vector3::new(x, yp, sz), 0);
                match voxel {
                    TreeBody::Leaf(c) => {
                        let color_temp = match_hsv_to_colorset(&colorset, &rgb2hsv(*c));
                        if color_temp != color { pass = false; break }
                    },
                    _ => { pass = false; break }
                }
            }
            if !pass { break }
            yp += 1;
        }

        while xp < len && (xp - x) < 200 {
            let mut pass = true;
            for sy in y..yp {
                for sz in z..zp {
                    let voxel = octree.get_mut_or_create(Vector3::new(xp, sy, sz), 0);
                    match voxel {
                        TreeBody::Leaf(c) => {
                            let color_temp = match_hsv_to_colorset(&colorset, &rgb2hsv(*c));
                            if color_temp != color { pass = false; break }
                        },
                        _ => { pass = false; break }
                    }
                }
                if !pass { break }
            }
            if !pass { break }
            xp += 1;
        }

        // Clear nodes
        // This cant be done during the loops above unless you keep track
        // of which nodes you have already deleted
        for sx in x..xp {
            for sy in y..yp {
                for sz in z..zp {
                    let voxel = octree.get_mut_or_create(Vector3::new(sx, sy, sz), 0);
                    *voxel = TreeBody::Empty;
                }
            }
        }

        let w = xp - x;
        let h = yp - y;
        let d = zp - z;

        write_data.bricks.push(
            brs::Brick {
                asset_name_index: 0,
                // Coordinates are rotated
                size: (5*w as u32, 5*d as u32, 2*h as u32),
                position: (
                    (5*w + 10*x) as i32,
                    (5*d + 10*z) as i32,
                    (2*h + 4*y) as i32
                ),
                direction: brs::Direction::ZPositive,
                rotation: brs::Rotation::Deg0,
                collision: true,
                visibility: true,
                material_index: 2,
                color: brs::ColorMode::Set(color as u32),
                owner_index: None
            }
        );
    }
}