use brs;
use tobj;

use std::fs::File;

use image::{RgbaImage};
use cgmath::Vector4;

mod octree;
mod intersect;
mod barycentric;
mod voxelize;
mod color;
mod simplify;

use octree::VoxelTree;
use voxelize::voxelize;
use simplify::*;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "textured-voxelizer", about = "Voxelizes OBJ files to create textured voxel models")]
struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    #[structopt(parse(from_os_str))]
    output: PathBuf,

    #[structopt(long, possible_values = &["lossy", "lossless"], default_value = "lossy")]
    simplify: String,

    #[structopt(short, long, default_value = "1")]
    scale: f32
}

fn main() {
    let opt = Opt::from_args();
    let mut octree = generate_octree(&opt);

    match opt.output.extension() {
        Some(extension) => {
            match extension.to_str() {
                Some("brs") => write_brs_data(&mut octree, opt.output, opt.simplify),
                // Implement new file types
                Some(extension) => panic!("Output file type {} is not supported", extension),
                None => panic!("Invalid output file type")
            }
        },
        None => panic!("Invalid output file type")
    }
}

fn generate_octree(opt: &Opt) -> VoxelTree<Vector4<u8>> {
    match opt.file.extension() {
        Some(extension) => {
            match extension.to_str() {
                Some("obj") => {}
                _ => panic!("Only input files of type obj are supported")
            }
        },
        None => panic!("Invalid input file type")
    };

    let file = match opt.file.canonicalize() {
        Err(e) => panic!("Error encountered when looking for file {:?}: {}", opt.file, e.to_string()),
        Ok(f) => f
    };

    println!("Importing model...");
    let (mut models, materials) = match tobj::load_obj(&file, true) {
        Err(e) => panic!("Error encountered when loading obj file: {}", e.to_string()),
        Ok(f) => f,
    };

    println!("Loading materials...");
    let mut material_images = Vec::<RgbaImage>::new();
    for material in materials {

        if material.diffuse_texture == "" {
            println!("\tMaterial {} does not have an associated diffuse texture", material.name);

            // Create mock texture from diffuse color
            let mut image = RgbaImage::new(1, 1);
            image.put_pixel(0, 0, image::Rgba([
                (material.diffuse[0] * 255.) as u8,
                (material.diffuse[1] * 255.) as u8,
                (material.diffuse[2] * 255.) as u8,
                (material.dissolve * 255.) as u8
            ]));

            material_images.push(image);
        } else {
            let image_path = file.canonicalize().unwrap().parent().unwrap().join(&material.diffuse_texture);
            println!("\tLoading diffuse texture for {} from: {:?}", material.name, image_path);

            let image = match image::open(&image_path) {
                Err(e) =>  panic!("Error encountered when loading {} texture file from {:?}: {}", &material.diffuse_texture, &image_path, e.to_string()),
                Ok(f) => f.into_rgba(),
            };
            material_images.push(image);
        }
    }

    println!("Voxelizing...");
    voxelize(&mut models, &material_images, opt.scale)
}

fn write_brs_data(mut octree: &mut VoxelTree::<Vector4::<u8>>, output: PathBuf, simplify_algo: String) {
    let blank_data = match File::open("blank.brs") {
        Err(e) => panic!("Error encountered when loading blank.brs file: {:}", e.to_string()),
        Ok(data) => data,
    };

    let mut write_data = brs::Reader::new(blank_data).unwrap().read_header1().unwrap().read_header2().unwrap().into_write_data().unwrap();
    write_data.bricks.clear();

    println!("Simplifying {:?}...", simplify_algo);
    if simplify_algo == "lossless" {
        simplify_lossless(&mut octree, &mut write_data);
    } else {
        simplify(&mut octree, &mut write_data);
    }

    // Write file
    println!("Writing file...");
    brs::write_save(&mut File::create(output).unwrap(), &write_data).unwrap();
}